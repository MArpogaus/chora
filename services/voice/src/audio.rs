//! Audio in and out through the capture and playback commands of the
//! settings: raw mono PCM16 at 24 kHz on a pipe.

use std::{
    io::{self, Read, Write},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    core::Input,
    gate::Frame,
    pcm::{self, Downsampler, RATE},
    vad::{self, Vad},
};

/// Samples per frame at 24 kHz: 32 ms, 512 samples at 16 kHz.
pub const FRAME: usize = vad::FRAME * 3 / 2;

fn spawn(cmd: &[String], stdin: Stdio, stdout: Stdio) -> io::Result<Child> {
    let (program, args) = cmd.split_first().ok_or(io::ErrorKind::InvalidInput)?;
    Command::new(program)
        .args(args)
        .stdin(stdin)
        .stdout(stdout)
        .spawn()
}

/// Turns raw PCM into frames with their speech probability and level.
pub struct Analyzer {
    vad: Vad,
    down: Downsampler,
    /// Frames slower than real time since the last log line.
    slow: u32,
}

impl Analyzer {
    pub fn new(vad: Vad) -> Self {
        Self {
            vad,
            down: Downsampler::default(),
            slow: 0,
        }
    }

    /// Starts again, for example after a new capture command.
    pub fn reset(&mut self) {
        self.vad.reset();
        self.down = Downsampler::default();
    }

    /// One frame of `FRAME` samples.
    ///
    /// # Errors
    ///
    /// The model fails.
    pub fn frame(&mut self, samples: Vec<i16>) -> Result<Frame, String> {
        let start = Instant::now();
        let low: [f32; vad::FRAME] = self
            .down
            .push(&samples)
            .try_into()
            .map_err(|_| "a frame of 768 samples")?;
        let prob = self.vad.probability(&low).map_err(|e| e.to_string())?;
        if start.elapsed() > Duration::from_millis(32) {
            self.slow += 1;
            if self.slow.is_power_of_two() {
                crate::log::warn(&format!(
                    "VAD slower than real time: {} frames, the last took {:?}",
                    self.slow,
                    start.elapsed()
                ));
            }
        }
        Ok(Frame { samples, prob })
    }
}

/// The capture command in a thread of its own; it sends each frame to
/// the core and starts the command again when it ends.
pub struct Mic {
    cmd: Arc<Mutex<Vec<String>>>,
    child: Arc<Mutex<Option<Child>>>,
    restarting: Arc<AtomicBool>,
}

impl Mic {
    pub fn start(cmd: Vec<String>, mut analyzer: Analyzer, tx: Sender<Input>) -> Self {
        let mic = Self {
            cmd: Arc::new(Mutex::new(cmd)),
            child: Arc::default(),
            restarting: Arc::default(),
        };
        let (cmd, slot, restarting) = (mic.cmd.clone(), mic.child.clone(), mic.restarting.clone());
        thread::spawn(move || {
            let mut backoff = Duration::from_millis(500);
            loop {
                let cmd = cmd.lock().expect("no panic with the lock").clone();
                let started = Instant::now();
                let ended = capture(&cmd, &slot, &mut analyzer, &tx);
                if matches!(ended, Err(Ended::CoreGone)) {
                    return;
                }
                analyzer.reset();
                if restarting.swap(false, Ordering::SeqCst) {
                    continue;
                }
                let Err(Ended::Mic(why)) = ended else {
                    continue;
                };
                crate::log::error(&format!("mic: {}: {why}", cmd.join(" ")));
                if tx.send(Input::MicDown).is_err() {
                    return;
                }
                // A command that ran a while starts a new episode.
                if started.elapsed() > Duration::from_secs(5) {
                    backoff = Duration::from_millis(500);
                }
                thread::sleep(backoff);
                backoff = (backoff * 2).min(Duration::from_secs(5));
            }
        });
        mic
    }

    /// Starts the capture again with a new command.
    pub fn restart(&self, cmd: Vec<String>) {
        *self.cmd.lock().expect("no panic with the lock") = cmd;
        self.restarting.store(true, Ordering::SeqCst);
        if let Some(child) = self.child.lock().expect("no panic with the lock").as_mut() {
            let _ = child.kill();
        }
    }
}

enum Ended {
    Mic(String),
    CoreGone,
}

/// Runs the command until it ends.
fn capture(
    cmd: &[String],
    slot: &Mutex<Option<Child>>,
    analyzer: &mut Analyzer,
    tx: &Sender<Input>,
) -> Result<(), Ended> {
    let mut child =
        spawn(cmd, Stdio::null(), Stdio::piped()).map_err(|e| Ended::Mic(e.to_string()))?;
    let mut out = child.stdout.take().expect("piped");
    *slot.lock().expect("no panic with the lock") = Some(child);
    let mut bytes = vec![0; FRAME * 2];
    let result = loop {
        if let Err(e) = out.read_exact(&mut bytes) {
            break Err(Ended::Mic(format!("the capture ended: {e}")));
        }
        let frame = match analyzer.frame(pcm::samples(&bytes)) {
            Ok(frame) => frame,
            Err(e) => break Err(Ended::Mic(e)),
        };
        if tx.send(Input::Frame(frame)).is_err() {
            break Err(Ended::CoreGone);
        }
    };
    if let Some(mut child) = slot.lock().expect("no panic with the lock").take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

/// The playback command: one process per sound, stopped at once on
/// request.
#[derive(Clone)]
pub struct Speaker {
    cmd: Vec<String>,
    child: Arc<Mutex<Option<Child>>>,
}

/// A sound in pieces: each `send` plays on after the piece before. The
/// process ends one second after the sound, when the sender is gone.
pub type Player = mpsc::Sender<Vec<i16>>;

impl Speaker {
    pub fn new(cmd: Vec<String>) -> Self {
        Self {
            cmd,
            child: Arc::default(),
        }
    }

    pub fn set_cmd(&mut self, cmd: Vec<String>) {
        self.cmd = cmd;
    }

    /// Plays PCM at 24 kHz and stops the sound before.
    ///
    /// # Errors
    ///
    /// The command does not start.
    pub fn play(&self, pcm: &[i16]) -> io::Result<()> {
        // The thread plays it even when the player is gone at once.
        let _ = self.stream()?.send(pcm.to_vec());
        Ok(())
    }

    /// Starts a sound in pieces and stops the sound before. A thread
    /// writes the pieces, and ends the process one second after the
    /// sound: a player on `PipeWire` may never exit by itself.
    ///
    /// # Errors
    ///
    /// The command does not start.
    pub fn stream(&self) -> io::Result<Player> {
        self.stop();
        let mut child = spawn(&self.cmd, Stdio::piped(), Stdio::null())?;
        let mut stdin = child.stdin.take().expect("piped");
        let id = child.id();
        *self.child.lock().expect("no panic with the lock") = Some(child);
        let slot = self.child.clone();
        let (tx, rx) = mpsc::channel::<Vec<i16>>();
        thread::spawn(move || {
            // When the sound written so far ends.
            let mut end = Instant::now();
            for pcm in rx {
                let samples = u64::try_from(pcm.len()).unwrap_or(u64::MAX);
                end = end.max(Instant::now())
                    + Duration::from_micros(samples * 1_000_000 / u64::from(RATE));
                // A stopped player closes the pipe.
                if stdin.write_all(&pcm::bytes(&pcm)).is_err() {
                    break;
                }
            }
            drop(stdin);
            let end = end + Duration::from_secs(1);
            loop {
                let mut slot = slot.lock().expect("no panic with the lock");
                let Some(child) = slot.as_mut().filter(|c| c.id() == id) else {
                    return;
                };
                if matches!(child.try_wait(), Ok(Some(_))) || Instant::now() >= end {
                    let _ = child.kill();
                    let _ = child.wait();
                    *slot = None;
                    return;
                }
                drop(slot);
                thread::sleep(Duration::from_millis(50));
            }
        });
        Ok(tx)
    }

    /// Stops the sound at once.
    pub fn stop(&self) {
        if let Some(mut child) = self.child.lock().expect("no panic with the lock").take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    fn sh(script: &str) -> Vec<String> {
        ["sh", "-c", script].map(Into::into).to_vec()
    }

    #[test]
    fn the_mic_sends_frames_and_restarts_a_command_that_ends() {
        let (tx, rx) = mpsc::channel();
        // Three frames of silence, then the command ends.
        let cmd = sh(&format!("head -c {} /dev/zero", FRAME * 2 * 3));
        let _mic = Mic::start(cmd, Analyzer::new(Vad::new().unwrap()), tx);
        let mut seen = Vec::new();
        while seen.len() < 5 {
            match rx.recv_timeout(Duration::from_secs(10)).unwrap() {
                Input::Frame(f) => seen.push(format!("frame {}", f.samples.len())),
                Input::MicDown => seen.push("down".into()),
                _ => unreachable!(),
            }
        }
        assert_eq!(
            seen,
            ["frame 768", "frame 768", "frame 768", "down", "frame 768"]
        );
    }

    #[test]
    fn the_speaker_writes_the_pcm_and_stop_ends_it() {
        let dir = std::env::temp_dir().join(format!("chora-voice-speaker-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("out.raw");
        let speaker = Speaker::new(sh(&format!("cat > {}", out.display())));
        speaker.play(&[1, 2, 3]).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while std::fs::read(&out).map_or(0, |b| b.len()) < 6 {
            assert!(Instant::now() < deadline, "the player got no PCM");
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(pcm::samples(&std::fs::read(&out).unwrap()), [1, 2, 3]);
        let slow = Speaker::new(sh("sleep 30"));
        slow.play(&[0; 10]).unwrap();
        let start = Instant::now();
        slow.stop();
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(slow.child.lock().unwrap().is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
