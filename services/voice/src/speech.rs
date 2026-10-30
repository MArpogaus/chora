//! The spoken reply: the words of the LLM split into sentences, one TTS
//! socket per sentence (the server does not split), and playback that
//! waits for a buffer, since the TTS can be slower than real time.

use std::time::{Duration, Instant};

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    audio::Speaker,
    chat::{Chat, Event, Report},
    config::Config,
    core::Reply,
    pcm::RATE,
    tts,
};

/// The first piece of a reply may end at a comma after this many
/// characters, so the first sound comes sooner.
const FIRST_PIECE: usize = 40;

/// Words before a period that end no sentence.
const ABBREVIATIONS: [&str; 10] = ["z", "b", "d", "h", "bzw", "usw", "ca", "etc", "vgl", "nr"];

/// Groups the streamed words of the LLM into sentences.
#[derive(Default)]
pub struct Splitter {
    buffer: String,
    /// A sentence was given out.
    started: bool,
}

impl Splitter {
    /// Adds words; answers the sentences that they end.
    pub fn push(&mut self, words: &str) -> Vec<String> {
        self.buffer.push_str(words);
        let mut out = Vec::new();
        while let Some(end) = self.boundary() {
            let rest = self.buffer.split_off(end);
            let sentence = std::mem::replace(&mut self.buffer, rest);
            self.give(&sentence, &mut out);
        }
        out
    }

    /// The rest, when the words end.
    pub fn finish(&mut self) -> Option<String> {
        let mut out = Vec::new();
        let rest = std::mem::take(&mut self.buffer);
        self.give(&rest, &mut out);
        out.pop()
    }

    fn give(&mut self, text: &str, out: &mut Vec<String>) {
        // The TTS would read the marks of markdown aloud.
        let text: String = text.chars().filter(|c| !"*#`_".contains(*c)).collect();
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            self.started = true;
            out.push(text);
        }
    }

    /// The end of the first sentence in the buffer: after `.!?:;` and a
    /// space, or at a line break.
    fn boundary(&self) -> Option<usize> {
        let text = &self.buffer;
        let mut chars = text.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c == '\n' {
                return Some(i + 1);
            }
            let next = chars.peek().map(|&(_, n)| n);
            if ".!?:;".contains(c) && next.is_some_and(char::is_whitespace) {
                let word = text[..i].rsplit(char::is_whitespace).next().unwrap_or("");
                let word = word.rsplit('.').next().unwrap_or(word).to_lowercase();
                let number = !word.is_empty() && word.chars().all(|c| c.is_ascii_digit());
                if c != '.' || !(number || ABBREVIATIONS.contains(&word.as_str())) {
                    return Some(i + 1);
                }
            }
        }
        if self.started {
            return None;
        }
        // The first piece may end at a comma.
        let comma = text
            .char_indices()
            .skip(FIRST_PIECE)
            .find(|&(i, c)| c == ',' && text[i + 1..].starts_with(char::is_whitespace));
        comma.map(|(i, _)| i + 1)
    }
}

/// What the speech of a reply did.
#[derive(Debug, Default)]
pub struct Spoken {
    /// The first sound went to the player.
    pub first_sound: Option<Instant>,
    /// From the request of the first sentence to its first PCM.
    pub tts_first: Option<Duration>,
    pub sentences: usize,
    pub errors: usize,
}

/// Speaks the sentences as they come, one TTS socket after the other,
/// and hands the PCM to `play`. Playback starts once
/// `playback_buffer_ms` of sound or the whole sentence is there, and
/// again so after it ran dry. Ends when the sound ends. A TTS error
/// skips its sentence.
pub async fn speak(
    config: &Config,
    mut sentences: UnboundedReceiver<String>,
    mut play: impl FnMut(Vec<i16>),
) -> Spoken {
    let buffer = usize::try_from(config.playback_buffer_ms * RATE / 1000).unwrap_or(usize::MAX);
    let mut spoken = Spoken::default();
    // When the sound written so far ends.
    let mut end = Instant::now();
    let mut write = |pcm: Vec<i16>, spoken: &mut Spoken| {
        let now = Instant::now();
        spoken.first_sound.get_or_insert(now);
        let samples = u64::try_from(pcm.len()).unwrap_or(u64::MAX);
        end = end.max(now) + Duration::from_micros(samples * 1_000_000 / u64::from(RATE));
        play(pcm);
        end
    };
    let mut sound_end = Instant::now();
    let mut held: Vec<i16> = Vec::new();
    while let Some(sentence) = sentences.recv().await {
        crate::log::info(&format!("say: {sentence}"));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let asked = Instant::now();
        let feed = async {
            let mut first = None;
            while let Some(event) = rx.recv().await {
                let tts::Event::Audio(pcm) = event else {
                    continue;
                };
                first.get_or_insert_with(|| asked.elapsed());
                // Held back while the sound ran dry.
                if Instant::now() >= sound_end {
                    held.extend(pcm);
                } else {
                    sound_end = write(pcm, &mut spoken);
                }
                if held.len() >= buffer {
                    sound_end = write(std::mem::take(&mut held), &mut spoken);
                }
            }
            first
        };
        let (result, first) = tokio::join!(tts::speak(config, &sentence, tx), feed);
        if !held.is_empty() {
            sound_end = write(std::mem::take(&mut held), &mut spoken);
        }
        if spoken.sentences == 0 {
            spoken.tts_first = first;
        }
        spoken.sentences += 1;
        if let Err(e) = result {
            crate::log::error(&format!("tts: {e}"));
            spoken.errors += 1;
        }
    }
    tokio::time::sleep_until(sound_end.into()).await;
    spoken
}

/// Runs a turn of `chat` and speaks its sentences and confirmation
/// prompts through `speaker`, none without it. `on` gets the events for
/// the core.
pub async fn reply(
    chat: &mut Chat,
    config: &Config,
    text: &str,
    speaker: Option<&Speaker>,
    on: &(impl Fn(Reply) + Sync),
) -> (Report, Spoken) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let turn = async {
        let tx = tx;
        let say = |sentence: String| {
            if speaker.is_some() {
                let _ = tx.send(sentence);
            }
        };
        chat.turn(text, &|event| match event {
            Event::Sentence(sentence) => say(sentence),
            Event::Confirm { prompt, answer } => {
                say(prompt.clone());
                on(Reply::Chat(Event::Confirm { prompt, answer }));
            }
            event => on(Reply::Chat(event)),
        })
        .await
    };
    let mut player = None;
    let play = |pcm| {
        let Some(speaker) = speaker else {
            return;
        };
        if player.is_none() {
            on(Reply::Sound);
            player = speaker
                .stream()
                .map_err(|e| crate::log::error(&format!("playback: {e}")))
                .ok();
        }
        if let Some(player) = &player {
            let _ = player.send(pcm);
        }
    };
    tokio::join!(turn, speak(config, rx, play))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake;

    fn split(pieces: &[&str]) -> Vec<String> {
        let mut splitter = Splitter::default();
        let mut out: Vec<String> = pieces.iter().flat_map(|p| splitter.push(p)).collect();
        out.extend(splitter.finish());
        out
    }

    #[test]
    fn the_splitter_ends_sentences_but_not_dates_and_abbreviations() {
        assert_eq!(
            split(&[
                "Hallo",
                " Marcel. Heute ist der 3. Okt",
                "ober, z. B. ",
                "ein Tag! Ok"
            ]),
            [
                "Hallo Marcel.",
                "Heute ist der 3. Oktober, z. B. ein Tag!",
                "Ok"
            ]
        );
        assert_eq!(
            split(&["Eins:\nzwei; **drei**"]),
            ["Eins:", "zwei;", "drei"]
        );
        assert_eq!(split(&["Ende."]), ["Ende."], "the last period ends it");
        assert_eq!(split(&["  \n "]), Vec::<String>::new());
    }

    #[test]
    fn the_first_piece_may_end_at_a_comma_after_forty_characters() {
        let long = "Ich habe das Fenster links neben das Terminal gelegt, und dann das andere, \
                    ganz ohne Mühe.";
        assert_eq!(
            split(&[long]),
            [
                "Ich habe das Fenster links neben das Terminal gelegt,",
                "und dann das andere, ganz ohne Mühe."
            ]
        );
        assert_eq!(split(&["Ja, gern."]), ["Ja, gern."], "a short start waits");
    }

    #[tokio::test]
    async fn playback_waits_for_a_buffer_or_the_end_of_the_sentence() {
        let server = fake::tts(fake::Tts::Speak).await;
        let config = Config {
            tts_url: server.url,
            playback_buffer_ms: 500,
            ..Config::default()
        };
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        // 60 ms of sound per character: 1.2 s and 0.18 s.
        tx.send("Zwanzig Zeichen hier.".into()).unwrap();
        tx.send("Ok.".into()).unwrap();
        drop(tx);
        let mut pieces = Vec::new();
        let start = Instant::now();
        let spoken = speak(&config, rx, |pcm| pieces.push(pcm.len())).await;
        assert_eq!(spoken.sentences, 2);
        assert_eq!(spoken.errors, 0);
        // The fake sends 2400 samples per message: the first write holds
        // 0.5 s (12000 samples) of it.
        assert!(pieces[0] >= 12_000, "{pieces:?}");
        assert_eq!(pieces.iter().sum::<usize>(), 24 * 1440);
        assert!(
            start.elapsed() >= Duration::from_millis(1300),
            "speak ends with the sound"
        );
    }

    #[tokio::test]
    async fn a_reply_speaks_its_sentences_and_the_question_of_a_confirmation() {
        let llm = fake::llm(vec![
            fake::Reply::Tool("echo", "{}"),
            fake::Reply::Text("Gut. Fertig."),
        ])
        .await;
        let tts = fake::tts(fake::Tts::Speak).await;
        let config = Config {
            llm_url: llm.url,
            tts_url: tts.url,
            confirm: vec![crate::config::Confirm {
                tool: "echo".into(),
                commands: Vec::new(),
                argv0: Vec::new(),
                method: None,
                prompt: "Wirklich?".into(),
            }],
            ..Config::default()
        };
        let (tools, _) = crate::tools::tests::fake(0);
        let mut chat = Chat::start(config.clone(), "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        let speaker = Speaker::new(["sh", "-c", "cat > /dev/null"].map(Into::into).to_vec());
        let events = std::sync::Mutex::new(Vec::new());
        let on = |reply| {
            let line = match reply {
                Reply::Chat(Event::Confirm { answer, .. }) => {
                    let _ = answer.send(true);
                    "confirm".into()
                }
                Reply::Chat(event) => format!("{event:?}"),
                Reply::Sound => "sound".into(),
                Reply::Done(_) => unreachable!(),
            };
            events.lock().unwrap().push(line);
        };
        let (report, spoken) = reply(&mut chat, &config, "Echo.", Some(&speaker), &on).await;
        assert_eq!(report.reply, "Gut. Fertig.");
        assert_eq!((spoken.sentences, spoken.errors), (3, 0));
        let texts: Vec<_> = tts
            .seen
            .lock()
            .unwrap()
            .iter()
            .filter_map(|m| m["text"].as_str().map(str::to_owned))
            .collect();
        assert_eq!(texts, ["Wirklich?", "Gut.", "Fertig."], "one socket each");
        let events = events.into_inner().unwrap();
        assert_eq!(events[..3], ["Thinking", "confirm", "Tool(\"echo\")"]);
        assert!(events.contains(&"sound".to_owned()), "{events:?}");
    }
}
