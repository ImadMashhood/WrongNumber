//! Local optional speech. Capture starts only after opt-in and push-to-talk.
use crate::{
    game::{Action, Screen, Session},
    ui::Actions,
};
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
#[derive(Resource, Default)]
pub struct ChatInput {
    pub draft: String,
    pub editing: bool,
    pub settings: bool,
    case: Option<(u32, u32)>,
}
#[derive(Clone)]
pub struct Clip {
    pub path: PathBuf,
    pub label: String,
}
enum ResultEvent {
    Devices(Vec<String>, Vec<String>),
    Captured(u32, u32, Result<Vec<u8>, String>),
    Text(u32, u32, Result<String, String>),
    Spoken(u32, u32, Result<Vec<u8>, String>),
    Status(String),
}
#[derive(Resource)]
pub struct Voice {
    pub enabled: bool,
    pub save_clips: bool,
    pub volume: f32,
    pub input: usize,
    pub output: usize,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub status: String,
    pub clips: Vec<Clip>,
    storage: PathBuf,
    pub recording: bool,
    pub speaking: bool,
    transcribing: bool,
    queued: Option<(u32, u32, String)>,
    stt: Option<PathBuf>,
    tts: Option<PathBuf>,
    stop: Arc<AtomicBool>,
    epoch: Arc<AtomicU64>,
    busy: Arc<AtomicBool>,
    tx: mpsc::SyncSender<ResultEvent>,
    rx: Mutex<mpsc::Receiver<ResultEvent>>,
    last: Option<(u32, u32, usize)>,
}
impl Default for Voice {
    fn default() -> Self {
        let args: Vec<_> = std::env::args().collect();
        let value = |f: &str| {
            args.iter()
                .position(|a| a == f)
                .and_then(|n| args.get(n + 1))
                .map(PathBuf::from)
        };
        let (tx, rx) = mpsc::sync_channel(12);
        let storage = PathBuf::from(".wrong-number/recordings");
        Self {
            enabled: false,
            save_clips: false,
            volume: 0.7,
            input: 0,
            output: 0,
            inputs: vec!["System default".into()],
            outputs: vec!["System default".into()],
            status: "Text ready. Microphone off. Hold V only after enabling it.".into(),
            clips: saved_clips(&storage),
            storage,
            recording: false,
            speaking: false,
            transcribing: false,
            queued: None,
            stt: value("--stt-command"),
            tts: value("--tts-command"),
            stop: Arc::new(AtomicBool::new(false)),
            epoch: Arc::new(AtomicU64::new(0)),
            busy: Arc::new(AtomicBool::new(false)),
            tx,
            rx: Mutex::new(rx),
            last: None,
        }
    }
}
impl Voice {
    pub fn local_action(&mut self, chat: &mut ChatInput, a: &Action) -> bool {
        match a {
            Action::ChatFocus => {
                chat.editing = !chat.editing;
            }
            Action::AudioSettings => {
                chat.settings = !chat.settings;
                if chat.settings {
                    let tx = self.tx.clone();
                    std::thread::spawn(move || {
                        let host = cpal::default_host();
                        let ins = host
                            .input_devices()
                            .map(|ds| {
                                ds.filter_map(|d| {
                                    d.description().ok().map(|v| v.name().to_string())
                                })
                                .take(32)
                                .collect()
                            })
                            .unwrap_or_default();
                        let outs = host
                            .output_devices()
                            .map(|ds| {
                                ds.filter_map(|d| {
                                    d.description().ok().map(|v| v.name().to_string())
                                })
                                .take(32)
                                .collect()
                            })
                            .unwrap_or_default();
                        let _ = tx.try_send(ResultEvent::Devices(ins, outs));
                    });
                }
            }
            Action::MicToggle => {
                self.enabled = !self.enabled;
                if !self.enabled {
                    self.stop.store(true, Ordering::Relaxed);
                    self.epoch.fetch_add(1, Ordering::Relaxed);
                }
                self.status = if self.enabled {
                    "Microphone enabled for hold-V capture; release to preview transcription."
                } else {
                    "Microphone off."
                }
                .into();
            }
            Action::SaveClips => {
                self.save_clips = !self.save_clips;
                self.status=if self.save_clips{"Local recording consent enabled. Only your mic and caller playback are saved; never other players' voices."}else{"Recording consent revoked; future clips will not be saved."}.into();
            }
            Action::InputDevice => {
                self.input = (self.input + 1) % self.inputs.len();
                self.stop.store(true, Ordering::Relaxed);
            }
            Action::OutputDevice => {
                self.output = (self.output + 1) % self.outputs.len();
                self.epoch.fetch_add(1, Ordering::Relaxed);
            }
            Action::Volume(up) => {
                self.volume = (self.volume + if *up { 0.1 } else { -0.1 }).clamp(0.0, 1.0);
                self.epoch.fetch_add(1, Ordering::Relaxed);
            }
            Action::DeleteClips => {
                self.save_clips = false;
                self.stop.store(true, Ordering::Relaxed);
                self.epoch.fetch_add(1, Ordering::Relaxed);
                if let Ok(files) = std::fs::read_dir(&self.storage) {
                    for f in files.flatten() {
                        if f.path().extension().is_some_and(|e| e == "wav") {
                            let _ = std::fs::remove_file(f.path());
                        }
                    }
                }
                self.clips.clear();
                self.status = "Local clips deleted; recording consent is off.".into();
            }
            Action::ReplayClip(n) => {
                if let Some(c) = self.clips.get(*n).cloned() {
                    self.play_file(c.path);
                }
            }
            _ => return false,
        }
        true
    }
    fn play_file(&mut self, path: PathBuf) {
        if self.busy.swap(true, Ordering::Relaxed) {
            self.status = "Audio busy; try again shortly.".into();
            return;
        }
        let epoch = self.epoch.fetch_add(1, Ordering::Relaxed) + 1;
        let token = self.epoch.clone();
        let busy = self.busy.clone();
        let tx = self.tx.clone();
        let out = self.output;
        let volume = self.volume;
        std::thread::spawn(move || {
            let r = std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|wav| play(&wav, out, volume, &token, epoch));
            if let Err(e) = r {
                let _ = tx.try_send(ResultEvent::Status(e));
            }
            busy.store(false, Ordering::Relaxed);
        });
    }
    fn speak(&mut self, shift: u32, id: u32, text: String) {
        self.epoch.fetch_add(1, Ordering::Relaxed);
        if self.busy.swap(true, Ordering::Relaxed) {
            self.queued = Some((shift, id, text));
            self.status = "Previous caller audio stopping; latest reply queued.".into();
            return;
        }
        let expected = self.epoch.load(Ordering::Relaxed);
        let epoch = self.epoch.clone();
        let busy = self.busy.clone();
        let tx = self.tx.clone();
        let command = self.tts.clone();
        let output = self.output;
        let volume = self.volume;
        std::thread::spawn(move || {
            let result = synthesize(command.as_deref(), &text, &epoch, expected);
            if epoch.load(Ordering::Relaxed) == expected {
                match result {
                    Ok(wav) => {
                        let _ = tx.try_send(ResultEvent::Spoken(shift, id, Ok(wav.clone())));
                        if let Err(e) = play(&wav, output, volume, &epoch, expected) {
                            let _ = tx.try_send(ResultEvent::Status(e));
                        }
                    }
                    Err(e) => {
                        let _ = tx.try_send(ResultEvent::Spoken(shift, id, Err(e)));
                    }
                }
            }
            busy.store(false, Ordering::Relaxed);
        });
    }
    fn start_capture(&mut self, shift: u32, id: u32) {
        if self.transcribing {
            self.status =
                "Wait for local transcription or disable the microphone to cancel.".into();
            return;
        }
        self.queued = None;
        if self.stt.is_none() {
            self.status =
                "No local STT adapter configured. Type your question or use an authored question."
                    .into();
            return;
        }
        self.stop = Arc::new(AtomicBool::new(false));
        let stop = self.stop.clone();
        let tx = self.tx.clone();
        let input = self.input;
        self.recording = true;
        self.status = "Listening while V is held (maximum 12 seconds)…".into();
        self.epoch.fetch_add(1, Ordering::Relaxed);
        std::thread::spawn(move || {
            let result = capture(input, &stop);
            let _ = tx.try_send(ResultEvent::Captured(shift, id, result));
        });
    }
    fn store(&mut self, shift: u32, id: u32, kind: &str, wav: &[u8]) {
        if !self.save_clips || !valid_wav(wav) {
            return;
        }
        let dir = self.storage.clone();
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
        let mut existing = saved_clips(&dir);
        while existing.len() >= 32 {
            let old = existing.remove(0);
            let _ = std::fs::remove_file(&old.path);
            self.clips.retain(|c| c.path != old.path);
        }
        let path = dir.join(format!(
            "shift{shift}-call{id}-{kind}-{}.wav",
            uuid::Uuid::new_v4()
        ));
        if private_write(&path, wav).is_ok() {
            self.clips.push(Clip {
                path,
                label: format!("Shift {shift} / call {id} / {kind}"),
            });
        }
        while self.clips.len() > 32 {
            let old = self.clips.remove(0);
            let _ = std::fs::remove_file(old.path);
        }
    }
}
fn saved_clips(dir: &Path) -> Vec<Clip> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|f| {
            f.file_type().is_ok_and(|t| t.is_file())
                && f.path().extension().is_some_and(|e| e == "wav")
        })
        .collect();
    files.sort_by_key(|f| f.metadata().and_then(|m| m.modified()).ok());
    files
        .into_iter()
        .map(|f| Clip {
            path: f.path(),
            label: f
                .file_name()
                .to_string_lossy()
                .split('-')
                .take(3)
                .collect::<Vec<_>>()
                .join(" / "),
        })
        .collect()
}
fn private_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(path)
        .and_then(|mut f| f.write_all(bytes))
        .map_err(|e| e.to_string())
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self, String> {
        let p = std::env::temp_dir().join(format!("wrong-number-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&p).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
        }
        Ok(Self(p))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run_command(
    mut cmd: std::process::Command,
    output: &Path,
    seconds: u64,
    token: &AtomicU64,
    expected: u64,
) -> Result<(), String> {
    if token.load(Ordering::Relaxed) != expected {
        return Err("Speech cancelled; text remains available".into());
    }
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut child = cmd
        .spawn()
        .map_err(|_| "Speech adapter not found; text remains available")?;
    let at = Instant::now();
    loop {
        if token.load(Ordering::Relaxed) != expected
            || at.elapsed() > Duration::from_secs(seconds)
            || std::fs::metadata(output).is_ok_and(|m| m.len() > 2_000_000)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Speech cancelled or timed out; text remains available".into());
        }
        match child.try_wait() {
            Ok(Some(s)) => {
                return if s.success() {
                    Ok(())
                } else {
                    Err("Speech adapter failed; text remains available".into())
                };
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(15)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Speech adapter failed".into());
            }
        }
    }
}
pub fn synthesize(
    program: Option<&Path>,
    text: &str,
    token: &AtomicU64,
    expected: u64,
) -> Result<Vec<u8>, String> {
    if text.len() > 512 {
        return Err("Caller speech exceeds limit".into());
    }
    let scratch = Scratch::new()?;
    let input = scratch.0.join("text.txt");
    let output = scratch.0.join("speech.wav");
    private_write(&input, text.as_bytes())?;
    let mut cmd = std::process::Command::new(program.unwrap_or_else(|| Path::new("espeak-ng")));
    if program.is_some() {
        cmd.arg(&input).arg(&output);
    } else {
        cmd.arg("-s")
            .arg("140")
            .arg("-w")
            .arg(&output)
            .arg("-f")
            .arg(&input);
    }
    run_command(cmd, &output, 8, token, expected)?;
    let bytes = std::fs::read(output).map_err(|_| "Speech adapter produced no WAV")?;
    if !valid_wav(&bytes) {
        return Err("Speech adapter returned an invalid WAV".into());
    }
    Ok(bytes)
}
pub fn transcribe(
    program: &Path,
    wav: &[u8],
    token: &AtomicU64,
    expected: u64,
) -> Result<String, String> {
    if !valid_wav(wav) {
        return Err("Invalid microphone WAV".into());
    }
    let scratch = Scratch::new()?;
    let input = scratch.0.join("microphone.wav");
    let output = scratch.0.join("transcript.txt");
    private_write(&input, wav)?;
    let mut cmd = std::process::Command::new(program);
    cmd.arg(&input).arg(&output);
    run_command(cmd, &output, 20, token, expected)?;
    let text = std::fs::read_to_string(output).map_err(|_| "No transcription produced")?;
    let text = text.trim().to_string();
    if text.is_empty() || text.len() > 256 || text.chars().any(|c| c.is_control()) {
        return Err("No usable speech recognized; type your question instead".into());
    }
    Ok(text)
}
pub fn valid_wav(wav: &[u8]) -> bool {
    wav.len() >= 44
        && wav.len() <= 2_000_000
        && wav.starts_with(b"RIFF")
        && wav.get(8..12) == Some(b"WAVE")
        && rodio::Decoder::try_from(std::io::Cursor::new(wav.to_vec())).is_ok()
}
fn play(
    wav: &[u8],
    output: usize,
    volume: f32,
    token: &AtomicU64,
    expected: u64,
) -> Result<(), String> {
    let host = cpal::default_host();
    let device = if output == 0 {
        host.default_output_device()
    } else {
        host.output_devices()
            .ok()
            .and_then(|mut ds| ds.nth(output - 1))
    }
    .ok_or("No speaker device; subtitles remain available")?;
    let sink = rodio::DeviceSinkBuilder::from_device(device)
        .and_then(|b| b.open_stream())
        .map_err(|_| "Speaker unavailable; subtitles remain available")?;
    let player = rodio::Player::connect_new(sink.mixer());
    player.set_volume(volume);
    let source = rodio::Decoder::try_from(std::io::Cursor::new(wav.to_vec()))
        .map_err(|_| "Invalid speech audio")?;
    player.append(source);
    let at = Instant::now();
    while !player.empty() {
        if token.load(Ordering::Relaxed) != expected || at.elapsed() > Duration::from_secs(25) {
            player.stop();
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    Ok(())
}
fn capture(input: usize, stop: &AtomicBool) -> Result<Vec<u8>, String> {
    let host = cpal::default_host();
    let device = if input == 0 {
        host.default_input_device()
    } else {
        host.input_devices()
            .ok()
            .and_then(|mut ds| ds.nth(input - 1))
    }
    .ok_or("No microphone device; text remains available")?;
    let supported = device
        .default_input_config()
        .map_err(|_| "Microphone unavailable")?;
    let config: cpal::StreamConfig = supported.clone().into();
    let data = Arc::new(Mutex::new(Vec::<f32>::new()));
    let failed = Arc::new(AtomicBool::new(false));
    let error = failed.clone();
    let format = supported.sample_format();
    macro_rules! stream {
        ($ty:ty,$convert:expr) => {{
            let data = data.clone();
            let channels = usize::from(config.channels);
            let cap = config.sample_rate as usize * 12;
            device.build_input_stream(
                &config,
                move |samples: &[$ty], _| {
                    if let Ok(mut target) = data.try_lock() {
                        for frame in samples.chunks(channels) {
                            if target.len() < cap {
                                target.push(($convert)(frame[0]));
                            }
                        }
                    }
                },
                move |_| {
                    error.store(true, Ordering::Relaxed);
                },
                None,
            )
        }};
    }
    let stream = match format {
        cpal::SampleFormat::F32 => stream!(f32, |v: f32| v),
        cpal::SampleFormat::I16 => stream!(i16, |v: i16| f32::from(v) / 32768.0),
        cpal::SampleFormat::U16 => stream!(u16, |v: u16| (f32::from(v) - 32768.0) / 32768.0),
        _ => return Err("Unsupported microphone format; text remains available".into()),
    }
    .map_err(|_| "Microphone cannot open")?;
    stream.play().map_err(|_| "Microphone cannot start")?;
    let at = Instant::now();
    while !stop.load(Ordering::Relaxed)
        && !failed.load(Ordering::Relaxed)
        && at.elapsed() < Duration::from_secs(12)
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(stream);
    if failed.load(Ordering::Relaxed) {
        return Err("Microphone disconnected".into());
    }
    let data = data.lock().unwrap();
    if data.len() < (config.sample_rate / 5) as usize {
        return Err("Hold V a little longer or type a question".into());
    }
    let pcm: Vec<i16> = (0..(data.len() * 16000 / config.sample_rate as usize))
        .map(|n| {
            let ix = n * config.sample_rate as usize / 16000;
            (data[ix].clamp(-1.0, 1.0) * 32767.0) as i16
        })
        .collect();
    Ok(wav_pcm(&pcm))
}
pub fn wav_pcm(samples: &[i16]) -> Vec<u8> {
    let size = (samples.len() * 2) as u32;
    let mut b = vec![];
    b.extend(b"RIFF");
    b.extend((36 + size).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(16000u32.to_le_bytes());
    b.extend(32000u32.to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(size.to_le_bytes());
    for s in samples {
        b.extend(s.to_le_bytes());
    }
    b
}
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut voice: ResMut<Voice>,
    mut chat: ResMut<ChatInput>,
    mut game: ResMut<Session>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut actions: ResMut<Actions>,
    check: Res<crate::playtest::LaunchCheck>,
) {
    let case = game.conversation.map(|id| (game.shift, id));
    if chat.case != case {
        chat.case = case;
        chat.draft.clear();
        chat.editing = false;
        voice.queued = None;
        voice.stop.store(true, Ordering::Relaxed);
        voice.epoch.fetch_add(1, Ordering::Relaxed);
    }
    if game.screen != Screen::Terminal {
        chat.editing = false;
    }
    let events: Vec<_> = voice.rx.lock().unwrap().try_iter().collect();
    for e in events {
        match e {
            ResultEvent::Devices(ins, outs) => {
                voice.inputs = std::iter::once("System default".into())
                    .chain(ins)
                    .collect();
                voice.outputs = std::iter::once("System default".into())
                    .chain(outs)
                    .collect();
                voice.input = voice.input.min(voice.inputs.len() - 1);
                voice.output = voice.output.min(voice.outputs.len() - 1);
            }
            ResultEvent::Captured(shift, id, result) => {
                voice.recording = false;
                if case != Some((shift, id)) || !voice.enabled {
                    continue;
                }
                match result {
                    Ok(wav) => {
                        voice.store(shift, id, "my-microphone", &wav);
                        if let Some(program) = voice.stt.clone() {
                            let tx = voice.tx.clone();
                            let token = voice.epoch.clone();
                            let expected = token.load(Ordering::Relaxed);
                            voice.status = "Transcribing locally…".into();
                            voice.transcribing = true;
                            std::thread::spawn(move || {
                                let result = transcribe(&program, &wav, &token, expected);
                                let _ = tx.try_send(ResultEvent::Text(shift, id, result));
                            });
                        }
                    }
                    Err(e) => voice.status = e,
                }
            }
            ResultEvent::Text(shift, id, result) => {
                voice.transcribing = false;
                if case != Some((shift, id)) || !voice.enabled {
                    continue;
                }
                match result {
                    Ok(text) => {
                        chat.draft = text;
                        chat.editing = true;
                        voice.status =
                            "Transcription ready. Edit it, then press Enter to send.".into();
                    }
                    Err(e) => voice.status = e,
                }
            }
            ResultEvent::Spoken(shift, id, result) => match result {
                Ok(wav) => {
                    voice.store(shift, id, "caller", &wav);
                    voice.status = "Caller audio / subtitles available.".into();
                }
                Err(e) => voice.status = e,
            },
            ResultEvent::Status(s) => voice.status = s,
        }
        game.dirty = true;
    }
    voice.speaking = voice.busy.load(Ordering::Relaxed);
    if !voice.speaking
        && !voice.recording
        && let Some((shift, id, text)) = voice.queued.take()
        && case == Some((shift, id))
        && voice.volume > 0.0
    {
        voice.speak(shift, id, text);
    }
    if chat.editing && (window.focused || check.automated()) && game.screen == Screen::Terminal {
        for e in keyboard.read() {
            if e.state != ButtonState::Pressed {
                continue;
            }
            match e.key_code {
                KeyCode::Enter => {
                    if let Some(id) = game.conversation
                        && !chat.draft.trim().is_empty()
                    {
                        actions.0.push(Action::Talk {
                            incident: id,
                            text: std::mem::take(&mut chat.draft),
                        });
                        chat.editing = false;
                    }
                }
                KeyCode::Backspace => {
                    chat.draft.pop();
                }
                _ => {
                    if let Some(t) = &e.text {
                        for c in t.chars().filter(|c| !c.is_control()) {
                            if chat.draft.len() + c.len_utf8() <= 256 {
                                chat.draft.push(c);
                            }
                        }
                    }
                }
            }
            game.dirty = true;
        }
    } else {
        keyboard.clear();
    }
    let active =
        case.is_some() && game.screen == Screen::Terminal && window.focused && !chat.editing;
    if !active || !keys.pressed(KeyCode::KeyV) {
        voice.stop.store(true, Ordering::Relaxed);
    }
    if !check.automated()
        && voice.enabled
        && active
        && keys.just_pressed(KeyCode::KeyV)
        && !voice.recording
    {
        let (shift, id) = case.unwrap();
        voice.start_capture(shift, id);
        game.dirty = true;
    }
    if let Some((shift, id)) = case
        && let Some(c) = game.dialogue.conversations.get(&id)
    {
        let marker = (shift, id, c.turns.len());
        if voice.last != Some(marker)
            && let Some(t) = c.turns.last()
        {
            voice.last = Some(marker);
            if !check.automated() && !std::env::args().any(|a| a == "--mute") && voice.volume > 0.0
            {
                voice.speak(shift, id, t.spoken_text.clone());
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn microphone_and_recording_require_explicit_opt_in() {
        let v = Voice::default();
        assert!(!v.enabled && !v.save_clips && !v.recording);
        assert!(v.clips.is_empty());
    }
    #[test]
    fn bounded_pcm_wav_roundtrips_and_invalid_audio_is_rejected() {
        let wav = wav_pcm(&vec![0; 16000]);
        assert!(valid_wav(&wav));
        assert_eq!(wav.len(), 32044);
        assert!(!valid_wav(b"invalid"));
        assert!(!valid_wav(&vec![0; 2_000_001]));
    }
    #[test]
    fn speech_cancellation_prevents_adapter_launch_work() {
        let token = AtomicU64::new(2);
        let r = run_command(
            std::process::Command::new("this-program-does-not-exist"),
            Path::new("missing.wav"),
            1,
            &token,
            1,
        );
        assert!(r.is_err());
        assert!(transcribe(Path::new("unused"), b"bad wav", &token, 2).is_err());
    }
    #[test]
    #[ignore = "Requires locally installed espeak-ng; run explicitly on the speech test host"]
    fn local_tts_returns_valid_audio() {
        let token = AtomicU64::new(0);
        let wav = synthesize(
            None,
            "They will not wake up and are barely breathing.",
            &token,
            0,
        )
        .unwrap();
        assert!(valid_wav(&wav));
    }
}
#[cfg(test)]
mod speech_pipeline_test {
    use super::*;
    #[test]
    #[ignore = "Requires local espeak-ng and WRONG_NUMBER_TEST_STT pointing to an installed adapter"]
    fn local_stt_adapter_recognizes_synthesized_question() {
        let adapter =
            std::env::var_os("WRONG_NUMBER_TEST_STT").expect("Set test adapter executable");
        let token = AtomicU64::new(0);
        let wav = synthesize(
            None,
            "Is your roommate breathing? Can you wake them up?",
            &token,
            0,
        )
        .unwrap();
        let text = transcribe(Path::new(&adapter), &wav, &token, 0)
            .unwrap()
            .to_lowercase();
        assert!(text.contains("breath") && text.contains("wake"), "{text}");
    }
}
#[cfg(test)]
mod recording_test {
    use super::*;
    #[test]
    fn consent_retention_and_delete_are_local_and_enforced() {
        let scratch = Scratch::new().unwrap();
        let mut v = Voice {
            storage: scratch.0.clone(),
            ..Default::default()
        };
        let mut chat = ChatInput::default();
        let wav = wav_pcm(&vec![0; 1600]);
        v.store(1, 1, "caller", &wav);
        assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 0);
        v.local_action(&mut chat, &Action::SaveClips);
        for n in 0..34 {
            v.store(1, n, "caller", &wav);
        }
        assert_eq!(v.clips.len(), 32);
        assert_eq!(saved_clips(&scratch.0).len(), 32);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&v.clips[0].path)
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        v.local_action(&mut chat, &Action::DeleteClips);
        assert!(!v.save_clips);
        assert!(v.clips.is_empty());
        assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 0);
    }
}
