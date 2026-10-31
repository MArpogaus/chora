//! The spoken reply: the words of the LLM split into sentences, one TTS
//! socket per sentence (the server does not split), and playback that
//! waits for a buffer, since the TTS can be slower than real time.

use std::time::{Duration, Instant};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

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

/// What the speech asks of the player.
#[derive(Debug, PartialEq)]
pub enum Out {
    /// PCM that plays from `at`, after the sound before.
    Pcm { at: Instant, pcm: Vec<i16> },
    /// A sentence: its sound starts at `at` and lasts `len`.
    Said {
        text: String,
        at: Instant,
        len: Duration,
    },
    /// Stops the sound at once: a barge-in.
    Stop,
    /// The TTS failed for a sentence.
    Failed,
}

/// A barge-in pauses the speech; the agent then resumes it from the cut
/// sentence, or after it with `skip`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Pause,
    Resume { skip: bool },
}

/// A piece of the TTS for the player.
enum Piece {
    Start(String),
    Pcm(Vec<i16>),
    Failed,
    Done,
}

/// A sentence in the player.
struct Sent {
    text: String,
    pcm: Vec<i16>,
    done: bool,
    /// The samples that went to the player.
    written: usize,
    /// When its sound starts.
    at: Option<Instant>,
    said: bool,
    skip: bool,
}

fn duration(samples: usize) -> Duration {
    let samples = u64::try_from(samples).unwrap_or(u64::MAX);
    Duration::from_micros(samples * 1_000_000 / u64::from(RATE))
}

/// Asks the TTS for each sentence, one socket after the other; a TTS
/// error skips its sentence. While the TTS is down, a sentence before
/// the end of its backoff skips without a try.
async fn fetch(
    config: &Config,
    mut sentences: UnboundedReceiver<String>,
    tx: UnboundedSender<Piece>,
    spoken: &mut Spoken,
) {
    while let Some(sentence) = sentences.recv().await {
        crate::log::info(&format!("say: {sentence}"));
        let _ = tx.send(Piece::Start(sentence.clone()));
        let (events, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let asked = Instant::now();
        let feed = async {
            let mut first = None;
            while let Some(event) = rx.recv().await {
                if let tts::Event::Audio(pcm) = event {
                    first.get_or_insert_with(|| asked.elapsed());
                    let _ = tx.send(Piece::Pcm(pcm));
                }
            }
            first
        };
        let tried = crate::health::ready("tts");
        let speak = async {
            if !tried {
                return Err("down, waiting for the backoff".to_owned());
            }
            tts::speak(config, &sentence, events).await
        };
        let (result, first) = tokio::join!(speak, feed);
        if let Err(e) = &result {
            crate::log::error(&format!("tts: {e}"));
            if tried {
                crate::health::down("tts", e);
            }
            spoken.errors += 1;
            let _ = tx.send(Piece::Failed);
        } else {
            crate::health::ok("tts");
        }
        let _ = tx.send(Piece::Done);
        if spoken.sentences == 0 {
            spoken.tts_first = first;
        }
        spoken.sentences += 1;
    }
}

/// The player side of `speak`.
struct Player<F> {
    out: F,
    sents: Vec<Sent>,
    /// The first sentence that is not all written.
    next: usize,
    /// When the sound written so far ends.
    end: Instant,
    buffer: usize,
    paused: bool,
    first_sound: Option<Instant>,
}

impl<F: FnMut(Out)> Player<F> {
    fn piece(&mut self, piece: Piece) {
        match piece {
            Piece::Start(text) => self.sents.push(Sent {
                text,
                pcm: Vec::new(),
                done: false,
                written: 0,
                at: None,
                said: false,
                skip: false,
            }),
            Piece::Pcm(pcm) => {
                if let Some(s) = self.sents.last_mut() {
                    s.pcm.extend(pcm);
                }
            }
            Piece::Done => {
                if let Some(s) = self.sents.last_mut() {
                    s.done = true;
                }
            }
            Piece::Failed => (self.out)(Out::Failed),
        }
    }

    /// Writes what may play: once `buffer` samples or the whole sentence
    /// are there, and again so after the sound ran dry.
    fn pump(&mut self) {
        if self.paused {
            return;
        }
        let now = Instant::now();
        let waiting: usize = self.sents[self.next..]
            .iter()
            .filter(|s| !s.skip)
            .map(|s| s.pcm.len() - s.written)
            .sum();
        let whole = self.sents.get(self.next).is_some_and(|s| s.done);
        if now >= self.end && waiting < self.buffer && !whole {
            return;
        }
        while let Some(s) = self.sents.get_mut(self.next) {
            if !s.skip && s.written < s.pcm.len() {
                let at = self.end.max(now);
                s.at.get_or_insert(at);
                let pcm = s.pcm[s.written..].to_vec();
                s.written = s.pcm.len();
                self.end = at + duration(pcm.len());
                self.first_sound.get_or_insert(now);
                (self.out)(Out::Pcm { at, pcm });
            }
            if !s.done {
                return;
            }
            if let (false, false, Some(at)) = (s.skip, s.said, s.at) {
                s.said = true;
                let (text, len) = (s.text.clone(), duration(s.pcm.len()));
                (self.out)(Out::Said { text, at, len });
            }
            self.next += 1;
        }
    }

    fn control(&mut self, control: Control) {
        match control {
            Control::Pause if !self.paused => {
                self.paused = true;
                (self.out)(Out::Stop);
                // The cut sentence and those after it play again.
                let now = Instant::now();
                let cut = self
                    .sents
                    .iter()
                    .position(|s| !s.skip && s.at.is_none_or(|at| at + duration(s.written) > now))
                    .unwrap_or(self.sents.len());
                for s in &mut self.sents[cut..] {
                    (s.written, s.at, s.said) = (0, None, false);
                }
                self.next = cut;
                self.end = now;
            }
            Control::Resume { skip } if self.paused => {
                self.paused = false;
                if skip && let Some(s) = self.sents.get_mut(self.next) {
                    s.skip = true;
                }
            }
            _ => {}
        }
    }

    fn written(&self) -> bool {
        self.next == self.sents.len()
    }
}

/// Speaks the sentences as they come, one TTS socket after the other,
/// and hands the sound to `out`. Playback starts once
/// `playback_buffer_ms` of sound or the whole sentence is there, and
/// again so after it ran dry. A pause stops the sound; the resume plays
/// the cut sentence again or skips it. Ends when the sound ends.
pub async fn speak(
    config: &Config,
    sentences: UnboundedReceiver<String>,
    mut control: UnboundedReceiver<Control>,
    out: impl FnMut(Out),
) -> Spoken {
    let mut spoken = Spoken::default();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut player = Player {
        out,
        sents: Vec::new(),
        next: 0,
        end: Instant::now(),
        buffer: usize::try_from(config.playback_buffer_ms * RATE / 1000).unwrap_or(usize::MAX),
        paused: false,
        first_sound: None,
    };
    let play = async {
        let (mut fetching, mut controlled) = (true, true);
        loop {
            player.pump();
            let ended = !fetching && player.written() && !player.paused;
            if ended && Instant::now() >= player.end {
                return;
            }
            tokio::select! {
                piece = rx.recv(), if fetching => match piece {
                    Some(piece) => player.piece(piece),
                    None => fetching = false,
                },
                c = control.recv(), if controlled => match c {
                    Some(c) => player.control(c),
                    None => controlled = false,
                },
                () = tokio::time::sleep_until(player.end.into()), if ended => {}
                // A pause whose control is gone: the reply ends with its
                // stop.
                else => std::future::pending::<()>().await,
            }
        }
    };
    tokio::join!(fetch(config, sentences, tx, &mut spoken), play);
    spoken.first_sound = player.first_sound;
    spoken
}

/// Runs a turn of `chat` and speaks its sentences and confirmation
/// prompts through the speaker of `voice`, none without it; the control
/// of `voice` pauses and resumes the speech. `on` gets the events for
/// the core. With `say`, the turn says `text` without the LLM, and the
/// history gets it as an answer.
pub async fn reply(
    chat: &mut Chat,
    config: &Config,
    (text, only_say): (&str, bool),
    voice: Option<(&Speaker, UnboundedReceiver<Control>)>,
    on: &(impl Fn(Reply) + Sync),
) -> (Report, Spoken) {
    let (speaker, control) = match voice {
        Some((speaker, control)) => (Some(speaker), control),
        None => (None, tokio::sync::mpsc::unbounded_channel().1),
    };
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let turn = async {
        let tx = tx;
        let say = |sentence: String| {
            if speaker.is_some() {
                let _ = tx.send(sentence);
            }
        };
        if only_say {
            return chat.said(text, say);
        }
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
    let out = |out| {
        let Some(speaker) = speaker else {
            return;
        };
        match out {
            Out::Pcm { at, pcm } => {
                on(Reply::Played {
                    at,
                    pcm: pcm.clone(),
                });
                if player.is_none() {
                    player = speaker
                        .stream()
                        .map_err(|e| crate::log::error(&format!("playback: {e}")))
                        .ok();
                }
                if let Some(player) = &player {
                    let _ = player.send(pcm);
                }
            }
            Out::Said { text, at, len } => on(Reply::Said { text, at, len }),
            Out::Failed => on(Reply::Failed),
            Out::Stop => {
                player = None;
                speaker.stop();
            }
        }
    };
    tokio::join!(turn, speak(config, rx, control, out))
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
        let (_control, nothing) = tokio::sync::mpsc::unbounded_channel();
        let spoken = speak(&config, rx, nothing, |out| {
            if let Out::Pcm { pcm, .. } = out {
                pieces.push(pcm.len());
            }
        })
        .await;
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

    /// Speaks two sentences, pauses at the first sound and resumes; the
    /// log of the player.
    async fn paused(config: &Config, skip: bool) -> Vec<String> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tx.send("Eins zwei drei.".into()).unwrap();
        tx.send("Vier.".into()).unwrap();
        drop(tx);
        let (control, controls) = tokio::sync::mpsc::unbounded_channel();
        let mut log = Vec::new();
        speak(config, rx, controls, |out| {
            if log.is_empty() {
                control.send(Control::Pause).unwrap();
                control.send(Control::Resume { skip }).unwrap();
            }
            log.push(match out {
                Out::Pcm { pcm, .. } => format!("pcm {}", pcm.len()),
                Out::Said { text, len, .. } => format!("said {text} {}", len.as_millis()),
                Out::Stop => "stop".into(),
                Out::Failed => "failed".into(),
            });
        })
        .await;
        log
    }

    #[tokio::test]
    async fn a_pause_stops_the_sound_and_the_resume_plays_the_cut_sentence_again() {
        let server = fake::tts(fake::Tts::Speak).await;
        let config = Config {
            tts_url: server.url,
            // The first write holds the whole first sentence, so the
            // pause cuts it however late the player sees the control.
            playback_buffer_ms: 900,
            ..Config::default()
        };
        // 60 ms of sound per character: 0.9 s and 0.3 s.
        let (eins, vier) = (15 * 1440, 5 * 1440);
        for (skip, after) in [(false, eins + vier), (true, vier)] {
            let log = paused(&config, skip).await;
            let stop = log.iter().position(|l| l == "stop").expect("a stop");
            let played: usize = log[stop..]
                .iter()
                .filter_map(|l| l.strip_prefix("pcm ")?.parse::<usize>().ok())
                .sum();
            assert_eq!(played, after, "skip={skip}: {log:?}");
            // The player may name the sentence once before the pause, as
            // it names a sentence when it writes it.
            let said: Vec<_> = log[stop..].iter().filter(|l| l.starts_with("said")).collect();
            let want: &[&str] = if skip {
                &["said Vier. 300"]
            } else {
                &["said Eins zwei drei. 900", "said Vier. 300"]
            };
            assert_eq!(said, want, "{log:?}");
        }
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
                Reply::Played { .. } => "sound".into(),
                Reply::Said { text, .. } => format!("said {text}"),
                Reply::Failed => "failed".into(),
                Reply::Done(_) => unreachable!(),
            };
            events.lock().unwrap().push(line);
        };
        let (_control, nothing) = tokio::sync::mpsc::unbounded_channel();
        let (report, spoken) = reply(
            &mut chat,
            &config,
            ("Echo.", false),
            Some((&speaker, nothing)),
            &on,
        )
        .await;
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
