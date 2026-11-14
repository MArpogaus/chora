//! Recordings of the sessions for replay (docs/voice.md section 12):
//! `mic.wav`, `played.wav` and `events.jsonl` in a directory per run.

use std::{
    fs::File,
    io::{self, BufWriter, Seek, SeekFrom, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::json;

use crate::pcm::{self, RATE};

/// A new file that only the user reads and writes (mode 0600): a
/// recording has the voice of the user.
fn private(path: &Path) -> io::Result<File> {
    File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}

/// A WAV file that grows; the header gets its length now and then and at
/// the end.
struct Wav {
    file: BufWriter<File>,
    samples: u32,
}

impl Wav {
    fn create(path: &Path) -> io::Result<Self> {
        let mut file = BufWriter::new(private(path)?);
        file.write_all(&pcm::wav_header(RATE, 0))?;
        Ok(Self { file, samples: 0 })
    }

    fn write(&mut self, samples: &[i16]) -> io::Result<()> {
        self.file.write_all(&pcm::bytes(samples))?;
        self.samples += u32::try_from(samples.len()).unwrap_or(u32::MAX);
        Ok(())
    }

    /// Writes the header with the length so far.
    fn sync(&mut self) -> io::Result<()> {
        let header = pcm::wav_header(RATE, self.samples * 2);
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&header)?;
        self.file.seek(SeekFrom::End(0))?;
        self.file.flush()
    }
}

/// The recording of one run.
pub struct Recorder {
    dir: PathBuf,
    mic: Wav,
    played: Wav,
    events: BufWriter<File>,
}

impl Recorder {
    /// Starts a recording in a new directory under `root`, named after
    /// the time (UTC). Only the user enters the directory (mode 0700).
    ///
    /// # Errors
    ///
    /// The directory or a file cannot be created.
    pub fn start(root: &Path) -> io::Result<Self> {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let dir = root.join(stamp(secs));
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&dir)?;
        Ok(Self {
            mic: Wav::create(&dir.join("mic.wav"))?,
            played: Wav::create(&dir.join("played.wav"))?,
            events: BufWriter::new(private(&dir.join("events.jsonl"))?),
            dir,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Appends a mic frame; about once a second, the headers get their
    /// lengths, so a crash keeps the recording.
    ///
    /// # Errors
    ///
    /// A file cannot be written.
    pub fn mic(&mut self, samples: &[i16]) -> io::Result<()> {
        self.mic.write(samples)?;
        if self.mic.samples % RATE < u32::try_from(samples.len()).unwrap_or(RATE) {
            self.sync()?;
        }
        Ok(())
    }

    /// Notes an event at the current mic sample.
    ///
    /// # Errors
    ///
    /// The file cannot be written.
    pub fn event(&mut self, event: &str) -> io::Result<()> {
        let line = json!({"at": self.mic.samples, "event": event});
        writeln!(self.events, "{line}")
    }

    /// Appends played sound that starts `ahead` samples after now, and
    /// notes where it starts.
    ///
    /// # Errors
    ///
    /// A file cannot be written.
    pub fn played(&mut self, samples: &[i16], ahead: usize) -> io::Result<()> {
        let line = json!({
            "at": self.mic.samples as usize + ahead,
            "event": "play",
            "from": self.played.samples,
            "len": samples.len(),
        });
        writeln!(self.events, "{line}")?;
        self.played.write(samples)
    }

    /// Writes the lengths and flushes all files.
    ///
    /// # Errors
    ///
    /// A file cannot be written.
    pub fn sync(&mut self) -> io::Result<()> {
        self.mic.sync()?;
        self.played.sync()?;
        self.events.flush()
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        if let Err(e) = self.sync() {
            crate::log::error(&format!("record: {}: {e}", self.dir.display()));
        }
    }
}

/// `YYYYMMDD-HHMMSS` in UTC of seconds since the epoch.
fn stamp(secs: u64) -> String {
    let (days, rest) = (secs / 86_400, secs % 86_400);
    // Days to the civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    let (h, m, s) = (rest / 3600, rest % 3600 / 60, rest % 60);
    format!("{year:04}{month:02}{day:02}-{h:02}{m:02}{s:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stamp_is_the_utc_date_and_time() {
        assert_eq!(stamp(0), "19700101-000000");
        assert_eq!(stamp(1_793_372_400), "20261030-150000");
        assert_eq!(stamp(951_782_400), "20000229-000000");
    }

    /// A recording has the voice of the user: only the user reads it.
    #[test]
    fn a_recording_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("chora-voice-mode-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let rec = Recorder::start(&root).unwrap();
        let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
        let modes = [
            mode(rec.dir()),
            mode(&rec.dir().join("mic.wav")),
            mode(&rec.dir().join("played.wav")),
            mode(&rec.dir().join("events.jsonl")),
        ];
        drop(rec);
        std::fs::remove_dir_all(root).unwrap();
        assert_eq!(modes, [0o700, 0o600, 0o600, 0o600]);
    }

    #[test]
    fn a_recording_keeps_mic_played_and_events() {
        let root = std::env::temp_dir().join(format!("chora-voice-rec-{}", std::process::id()));
        let mut rec = Recorder::start(&root).unwrap();
        let dir = rec.dir().to_owned();
        rec.mic(&[1; 768]).unwrap();
        rec.event("onset").unwrap();
        rec.played(&[7; 100], 0).unwrap();
        rec.mic(&[2; 768]).unwrap();
        // A crash before the end: the header has no length yet.
        rec.mic.file.flush().unwrap();
        let (_, cut) = pcm::read_wav(&dir.join("mic.wav")).unwrap();
        assert_eq!(
            cut.len(),
            2 * 768,
            "a reader takes a file without its length"
        );
        drop(rec);
        let (rate, mic) = pcm::read_wav(&dir.join("mic.wav")).unwrap();
        assert_eq!((rate, mic.len(), mic[800]), (RATE, 1536, 2));
        assert_eq!(pcm::read_wav(&dir.join("played.wav")).unwrap().1, [7; 100]);
        let events = std::fs::read_to_string(dir.join("events.jsonl")).unwrap();
        assert_eq!(
            events,
            "{\"at\":768,\"event\":\"onset\"}\n{\"at\":768,\"event\":\"play\",\"from\":0,\"len\":100}\n"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
