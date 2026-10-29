//! Local front end. The player who creates a room is its simulation authority.
use crate::{
    game::{Screen, Session},
    network::Network,
    ui,
    voice::{ChatInput, Voice},
};
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button},
    window::{MonitorSelection, PresentMode, WindowMode},
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Mutex};

fn settings_path() -> String {
    let args: Vec<_> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--settings-file")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| ".wrong-number/settings.json".into())
}
const LABELS: [&str; 8] = [
    "Forward",
    "Back",
    "Left",
    "Right",
    "Sprint",
    "Interact / sit",
    "Stand up",
    "Push to talk",
];
#[derive(Resource, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Preferences {
    pub version: u32,
    pub text_scale: f32,
    pub subtitles: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub sensitivity: f32,
    pub invert_y: bool,
    pub keys: [String; 8],
    pub fullscreen: bool,
    pub vsync: bool,
    pub show_fps: bool,
    pub voice_mode: u8,
    pub volume: f32,
    pub ring_volume: f32,
    pub ambient_volume: f32,
    pub gate_db: i32,
    pub listen_seconds: u32,
    pub input_name: String,
    pub output_name: String,
    pub review_speech: bool,
    pub name: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            version: 1,
            text_scale: 1.0,
            subtitles: true,
            high_contrast: false,
            reduced_motion: false,
            sensitivity: 1.0,
            invert_y: false,
            keys: [
                "KeyW",
                "KeyS",
                "KeyA",
                "KeyD",
                "ShiftLeft",
                "KeyE",
                "Space",
                "KeyV",
            ]
            .map(str::to_string),
            fullscreen: false,
            vsync: true,
            show_fps: false,
            voice_mode: 0,
            volume: 0.7,
            ring_volume: 0.6,
            ambient_volume: 0.18,
            gate_db: -48,
            listen_seconds: 12,
            input_name: "System default".into(),
            output_name: "System default".into(),
            review_speech: false,
            name: "Dispatcher".into(),
        }
    }
}
fn key_label(s: &str) -> &str {
    match s {
        "ShiftLeft" => "Left Shift",
        "ShiftRight" => "Right Shift",
        "ControlLeft" => "Left Ctrl",
        "ControlRight" => "Right Ctrl",
        "AltLeft" => "Left Alt",
        "AltRight" => "Right Alt",
        _ => s
            .strip_prefix("Key")
            .or_else(|| s.strip_prefix("Digit"))
            .unwrap_or(s),
    }
}
fn on_off(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}
fn valid_binding(s: &str) -> bool {
    (s.len() == 4 && s.starts_with("Key") && s.as_bytes()[3].is_ascii_uppercase())
        || (s.len() == 6 && s.starts_with("Digit") && s.as_bytes()[5].is_ascii_digit())
        || matches!(
            s,
            "Space"
                | "Tab"
                | "ShiftLeft"
                | "ShiftRight"
                | "ControlLeft"
                | "ControlRight"
                | "AltLeft"
                | "AltRight"
                | "ArrowUp"
                | "ArrowDown"
                | "ArrowLeft"
                | "ArrowRight"
                | "F1"
                | "F2"
                | "F4"
                | "F5"
                | "F6"
                | "F7"
                | "F8"
                | "F9"
                | "F10"
                | "F11"
                | "F12"
        )
}
impl Preferences {
    pub fn load() -> Self {
        let mut p: Self = std::fs::read(settings_path())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        p.text_scale = if p.text_scale.is_finite() {
            p.text_scale.clamp(1.0, 1.3)
        } else {
            1.0
        };
        p.sensitivity = if p.sensitivity.is_finite() {
            p.sensitivity.clamp(0.2, 3.0)
        } else {
            1.0
        };
        p.volume = if p.volume.is_finite() {
            p.volume.clamp(0.0, 1.0)
        } else {
            0.7
        };
        p.ring_volume = if p.ring_volume.is_finite() {
            p.ring_volume.clamp(0.0, 1.0)
        } else {
            0.6
        };
        p.ambient_volume = if p.ambient_volume.is_finite() {
            p.ambient_volume.clamp(0.0, 1.0)
        } else {
            0.18
        };
        p.gate_db = p.gate_db.clamp(-90, 0);
        p.listen_seconds = p.listen_seconds.clamp(3, 12);
        p.voice_mode = p.voice_mode.min(2);
        // Unknown or duplicate bindings must not leave movement unusable.
        let defaults = Self::default();
        for i in 0..8 {
            if !valid_binding(&p.keys[i]) || p.keys[..i].contains(&p.keys[i]) {
                p.keys = defaults.keys;
                break;
            }
        }
        p
    }
    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(&settings_path())
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(format!("{}.tmp", settings_path()), bytes).map_err(|e| e.to_string())?;
        std::fs::rename(format!("{}.tmp", settings_path()), settings_path())
            .map_err(|e| e.to_string())
    }
    pub fn held(&self, keys: &ButtonInput<KeyCode>, index: usize) -> bool {
        keys.get_pressed()
            .any(|k| format!("{k:?}") == self.keys[index])
    }
    pub fn pressed(&self, keys: &ButtonInput<KeyCode>, index: usize) -> bool {
        keys.get_just_pressed()
            .any(|k| format!("{k:?}") == self.keys[index])
    }
    fn apply_voice(&self, v: &mut Voice) {
        v.enabled = self.voice_mode != 2;
        v.push_to_talk = self.voice_mode == 1;
        v.volume = self.volume;
        v.gate_db = self.gate_db;
        v.listen_seconds = self.listen_seconds;
        v.review_speech = self.review_speech;
        v.preferred_input = self.input_name.clone();
        v.preferred_output = self.output_name.clone();
    }
    pub fn capture_voice(&mut self, v: &Voice) {
        self.voice_mode = if !v.enabled {
            2
        } else if v.push_to_talk {
            1
        } else {
            0
        };
        self.volume = v.volume;
        self.gate_db = v.gate_db;
        self.listen_seconds = v.listen_seconds;
        self.input_name = v.inputs[v.input].clone();
        self.output_name = v.outputs[v.output].clone();
        self.review_speech = v.review_speech;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Main,
    Host,
    Join,
    Code,
    Settings,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Name,
    Address,
    Port,
    Advertise,
    Code,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Page(Page),
    Field(Field),
    Host,
    Join,
    Solo,
    Back,
    Settings,
    Leave,
    Quit,
    Sensitivity(bool),
    TextScale,
    Subtitles,
    Contrast,
    Motion,
    Invert,
    RingVolume(bool),
    AmbientVolume(bool),
    Bind(usize),
    Fullscreen,
    Vsync,
    Fps,
    Mode,
    Reset,
    CopyCode,
    CopyAddress,
}
#[derive(Component)]
pub struct MenuButton(pub MenuAction);
#[derive(Resource)]
pub struct Frontend {
    pub visible: bool,
    pub page: Page,
    pub name: String,
    pub address: String,
    pub port: String,
    pub advertise: String,
    pub code: String,
    pub hosted_code: String,
    pub status: String,
    pub focus: Option<Field>,
    pub rebind: Option<usize>,
    pub actions: Vec<MenuAction>,
    pub dirty: bool,
    pub(crate) in_game: bool,
    return_screen: Screen,
    clipboard: Option<Mutex<arboard::Clipboard>>,
    pending: Option<Mutex<std::sync::mpsc::Receiver<Result<Network, String>>>>,
}
impl Frontend {
    pub fn new(show: bool, name: String) -> Self {
        Self {
            visible: show,
            page: Page::Main,
            name,
            address: "127.0.0.1:47000".into(),
            port: "47000".into(),
            advertise: String::new(),
            code: String::new(),
            hosted_code: String::new(),
            status: String::new(),
            focus: None,
            rebind: None,
            actions: vec![],
            dirty: true,
            in_game: !show,
            return_screen: Screen::Office,
            pending: None,
            clipboard: arboard::Clipboard::new().ok().map(Mutex::new),
        }
    }
    fn field(&mut self) -> Option<&mut String> {
        match self.focus? {
            Field::Name => Some(&mut self.name),
            Field::Address => Some(&mut self.address),
            Field::Port => Some(&mut self.port),
            Field::Advertise => Some(&mut self.advertise),
            Field::Code => Some(&mut self.code),
        }
    }
}
// Versioned IPv4/IPv6 endpoint code with a typo checksum. This is an address,
// not a password, a public matchmaking registration or a NAT traversal service.
pub fn encode_code(addr: SocketAddr) -> String {
    let mut bytes = match addr.ip() {
        std::net::IpAddr::V4(ip) => {
            let mut b = vec![4];
            b.extend(ip.octets());
            b
        }
        std::net::IpAddr::V6(ip) => {
            let mut b = vec![6];
            b.extend(ip.octets());
            b
        }
    };
    bytes.extend(addr.port().to_be_bytes());
    let sum = checksum(&bytes);
    bytes.extend(sum.to_be_bytes());
    format!(
        "WN1-{}",
        bytes
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<String>()
            .as_bytes()
            .chunks(6)
            .map(|b| std::str::from_utf8(b).unwrap())
            .collect::<Vec<_>>()
            .join("-")
    )
}
fn checksum(bytes: &[u8]) -> u16 {
    bytes
        .iter()
        .fold(0x811cu16, |s, b| (s ^ u16::from(*b)).wrapping_mul(257))
}
pub fn decode_code(code: &str) -> Result<SocketAddr, String> {
    let code = code.trim().to_ascii_uppercase();
    let raw = code
        .strip_prefix("WN1-")
        .ok_or("Use a complete WN1 join code")?
        .replace('-', "");
    if ![18, 42].contains(&raw.len()) || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid join code length or characters".into());
    }
    let bytes: Vec<u8> = (0..raw.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&raw[i..i + 2], 16).unwrap())
        .collect();
    let n = bytes.len();
    if checksum(&bytes[..n - 2]) != u16::from_be_bytes([bytes[n - 2], bytes[n - 1]]) {
        return Err("Join code checksum failed; check the copied code".into());
    }
    let ip = match (bytes[0], n) {
        (4, 9) => std::net::IpAddr::V4(std::net::Ipv4Addr::new(
            bytes[1], bytes[2], bytes[3], bytes[4],
        )),
        (6, 21) => {
            let mut octets = [0; 16];
            octets.copy_from_slice(&bytes[1..17]);
            std::net::IpAddr::V6(octets.into())
        }
        _ => return Err("Unsupported join code".into()),
    };
    let addr = SocketAddr::new(ip, u16::from_be_bytes([bytes[n - 4], bytes[n - 3]]));
    if addr.port() == 0 || addr.ip().is_unspecified() || addr.ip().is_multicast() {
        return Err("Join code has an unusable address".into());
    }
    Ok(addr)
}
pub fn setup(
    mut prefs: ResMut<Preferences>,
    mut voice: ResMut<Voice>,
    mut chat: ResMut<ChatInput>,
    mut window: Single<&mut Window>,
    mut frames: ResMut<crate::performance::Frames>,
    check: Res<crate::playtest::LaunchCheck>,
) {
    if check.automated() {
        *prefs = Preferences::default();
        return;
    }
    prefs.apply_voice(&mut voice);
    voice.local_action(&mut chat, &crate::game::Action::AudioSettings);
    chat.settings = false;
    apply_display(&prefs, &mut window, &mut frames);
}
fn apply_display(p: &Preferences, w: &mut Window, f: &mut crate::performance::Frames) {
    w.mode = if p.fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    };
    w.present_mode = if p.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    f.show = p.show_fps;
}
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut menu: ResMut<Frontend>,
    mut prefs: ResMut<Preferences>,
    mut net: ResMut<Network>,
    mut game: ResMut<Session>,
    mut voice: ResMut<Voice>,
    mut chat: ResMut<ChatInput>,
    mut keyboard: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window>,
    mut frames: ResMut<crate::performance::Frames>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    if let Some(result) = menu
        .pending
        .as_ref()
        .and_then(|rx| rx.lock().unwrap().try_recv().ok())
    {
        menu.pending = None;
        match result {
            Ok(mut next) => {
                *game = next.initial_game();
                next.correction = Some(crate::room::Pose::spawn(1));
                *net = next;
                menu.visible = false;
                menu.in_game = true;
                menu.focus = None;
                game.dirty = true;
            }
            Err(e) => {
                menu.status = format!(
                    "Could not connect: {e}. Check the address, firewall and TCP port forwarding."
                )
            }
        }
        menu.dirty = true;
    }
    let mut save = false;
    for a in std::mem::take(&mut menu.actions) {
        menu.dirty = true;
        match a {
            MenuAction::Page(p) => {
                menu.pending = None;
                menu.page = p;
                menu.focus = None;
                menu.status.clear();
            }
            MenuAction::Field(f) => {
                menu.focus = Some(f);
                menu.rebind = None;
            }
            MenuAction::Settings => {
                menu.return_screen = game.screen;
                if !menu.visible {
                    game.screen = Screen::Pause;
                }
                menu.visible = true;
                menu.page = Page::Settings;
                chat.settings = true;
            }
            MenuAction::Back => {
                menu.pending = None;
                menu.rebind = None;
                menu.focus = None;
                if menu.in_game {
                    menu.visible = false;
                    game.screen = menu.return_screen;
                    game.dirty = true;
                } else {
                    menu.page = Page::Main;
                }
            }
            MenuAction::Leave => {
                menu.pending = None;
                *net = Network::from_options(vec![]).unwrap();
                *game = net.initial_game();
                game.lobby();
                menu.visible = true;
                menu.in_game = false;
                menu.page = Page::Main;
                menu.hosted_code.clear();
                voice.cancel_capture();
                voice.cancel_playback();
            }
            MenuAction::Quit => {
                exit.write(bevy::app::AppExit::Success);
            }
            MenuAction::Host | MenuAction::Join | MenuAction::Solo => {
                if menu.pending.is_some() {
                    continue;
                }
                if menu.name.trim().is_empty() || menu.name.chars().count() > 24 {
                    menu.status = "Enter a dispatcher name (1–24 characters).".into();
                    continue;
                }
                let mut args = vec![
                    "wrong-number".into(),
                    "--name".into(),
                    menu.name.trim().into(),
                ];
                let launch = match a {
                    MenuAction::Host => match menu.port.parse::<u16>().ok().filter(|p| *p > 0) {
                        None => Err("Enter a TCP port from 1 to 65535".to_string()),
                        Some(port) => {
                            let advertised = menu.advertise.trim().parse::<std::net::IpAddr>();
                            match advertised {
                                    Ok(ip) if ip.is_ipv4() && !ip.is_unspecified() && !ip.is_multicast() => {
                                        menu.hosted_code = encode_code(SocketAddr::new(ip, port));
                                        args.extend(["--host".into(), format!("0.0.0.0:{port}")]);
                                        Ok(())
                                    }
                                    _ => Err("Enter the public IPv4 address friends will connect to, or your LAN IPv4 for local play.".into()),
                                }
                        }
                    },
                    MenuAction::Join => {
                        let address = if menu.page == Page::Code {
                            decode_code(&menu.code).map(|a| a.to_string())
                        } else {
                            menu.address
                                .trim()
                                .parse::<SocketAddr>()
                                .map(|a| a.to_string())
                                .map_err(|_| {
                                    "Enter an IP:port, for example 192.168.1.20:47000".to_string()
                                })
                        };
                        address.map(|address| {
                            args.extend([
                                "--join".into(),
                                address,
                                "--resume".into(),
                                format!(".wrong-number/menu-room-{}.json", uuid::Uuid::new_v4()),
                            ]);
                        })
                    }
                    _ => Ok(()),
                };
                if let Err(e) = launch {
                    menu.status = e;
                    continue;
                }
                if a != MenuAction::Join {
                    let old: Vec<_> = std::env::args().collect();
                    if let Some(i) = old.iter().position(|s| s == "--caller-service")
                        && let Some(v) = old.get(i + 1)
                    {
                        args.extend(["--caller-service".into(), v.clone()]);
                    }
                }
                prefs.name = menu.name.trim().into();
                save = true;
                let (tx, rx) = std::sync::mpsc::sync_channel(1);
                menu.pending = Some(Mutex::new(rx));
                menu.status = if a == MenuAction::Host {
                    "Creating your room…"
                } else {
                    "Connecting…"
                }
                .into();
                std::thread::spawn(move || {
                    let _ = tx.send(Network::from_options(args));
                });
            }
            MenuAction::TextScale => {
                prefs.text_scale = if prefs.text_scale >= 1.29 {
                    1.0
                } else {
                    prefs.text_scale + 0.1
                };
                save = true;
            }
            MenuAction::Subtitles => {
                prefs.subtitles = !prefs.subtitles;
                save = true;
            }
            MenuAction::Contrast => {
                prefs.high_contrast = !prefs.high_contrast;
                save = true;
            }
            MenuAction::Motion => {
                prefs.reduced_motion = !prefs.reduced_motion;
                save = true;
            }
            MenuAction::Sensitivity(up) => {
                prefs.sensitivity =
                    (prefs.sensitivity + if up { 0.1 } else { -0.1 }).clamp(0.2, 3.0);
                save = true;
            }
            MenuAction::AmbientVolume(up) => {
                prefs.ambient_volume =
                    (prefs.ambient_volume + if up { 0.05 } else { -0.05 }).clamp(0.0, 1.0);
                save = true;
            }
            MenuAction::RingVolume(up) => {
                prefs.ring_volume =
                    (prefs.ring_volume + if up { 0.1 } else { -0.1 }).clamp(0.0, 1.0);
                save = true;
            }
            MenuAction::Invert => {
                prefs.invert_y = !prefs.invert_y;
                save = true;
            }
            MenuAction::Bind(i) => {
                menu.rebind = Some(i);
                menu.focus = None;
                menu.status = "Press a key. Escape cancels; duplicate keys swap bindings.".into();
            }
            MenuAction::Fullscreen => {
                prefs.fullscreen = !prefs.fullscreen;
                save = true;
            }
            MenuAction::Vsync => {
                prefs.vsync = !prefs.vsync;
                save = true;
            }
            MenuAction::Fps => {
                prefs.show_fps = !prefs.show_fps;
                save = true;
            }
            MenuAction::Mode => {
                prefs.voice_mode = (prefs.voice_mode + 1) % 3;
                voice.cancel_capture();
                prefs.apply_voice(&mut voice);
                save = true;
            }
            MenuAction::Reset => {
                *prefs = Preferences::default();
                voice.cancel_capture();
                prefs.apply_voice(&mut voice);
                voice.input = 0;
                voice.output = 0;
                save = true;
            }
            MenuAction::CopyCode | MenuAction::CopyAddress => {
                let value = if a == MenuAction::CopyCode {
                    menu.hosted_code.clone()
                } else {
                    decode_code(&menu.hosted_code)
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| net.address.clone())
                };
                menu.status = match menu
                    .clipboard
                    .as_ref()
                    .ok_or(arboard::Error::ClipboardNotSupported)
                    .and_then(|c| c.lock().unwrap().set_text(value))
                {
                    Ok(_) => "Copied to clipboard".into(),
                    Err(_) => {
                        "Clipboard unavailable; select the code from the host information.".into()
                    }
                };
                game.notice = menu.status.clone();
                game.dirty = true;
            }
        }
    }
    if menu.visible && window.focused {
        for e in keyboard.read().filter(|e| e.state == ButtonState::Pressed) {
            if let Some(i) = menu.rebind {
                if !matches!(e.key_code, KeyCode::Escape | KeyCode::F3) {
                    let value = format!("{:?}", e.key_code);
                    if !valid_binding(&value) {
                        menu.status = "Use a letter, number, arrow, Space, Tab, Shift, Control, Alt or F1–F12 (F3 is reserved).".into();
                        continue;
                    }
                    if let Some(other) = prefs.keys.iter().position(|s| *s == value) {
                        prefs.keys.swap(i, other);
                    } else {
                        prefs.keys[i] = value;
                    }
                    save = true;
                }
                menu.rebind = None;
                menu.status.clear();
                menu.dirty = true;
                continue;
            }
            if e.key_code == KeyCode::Escape {
                menu.actions.push(MenuAction::Back);
                continue;
            }
            let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
            if let Some(value) = menu.field() {
                if ctrl && e.key_code == KeyCode::KeyA {
                    value.clear();
                } else if ctrl && e.key_code == KeyCode::KeyV {
                    if let Ok(text) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                        value.extend(
                            text.chars()
                                .filter(|c| !c.is_control())
                                .take(128 - value.chars().count().min(128)),
                        );
                    }
                } else if e.key_code == KeyCode::Backspace {
                    value.pop();
                } else if !ctrl && let Some(t) = &e.text {
                    for c in t.chars().filter(|c| !c.is_control()) {
                        if value.len() + c.len_utf8() <= 128 {
                            value.push(c);
                        }
                    }
                }
                menu.dirty = true;
            }
        }
    } else {
        keyboard.clear();
    }
    if save {
        apply_display(&prefs, &mut window, &mut frames);
        menu.status = prefs
            .save()
            .map(|_| "Settings saved".into())
            .unwrap_or_else(|e| format!("Settings could not be saved: {e}"));
        menu.dirty = true;
    }
}
fn text(p: &mut ChildSpawnerCommands, s: impl Into<String>, size: f32, color: Color) {
    p.spawn((
        Text::new(s),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    ));
}
pub fn button(p: &mut ChildSpawnerCommands, label: impl Into<String>, action: MenuAction) {
    p.spawn((
        Node {
            min_height: px(37),
            padding: UiRect::all(px(10)),
            margin: UiRect::bottom(px(5)),
            flex_shrink: 0.0,
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.10, 0.22, 0.26)),
        Button,
        Hovered::default(),
        MenuButton(action.clone()),
    ))
    .observe(move |_: On<Activate>, mut menu: ResMut<Frontend>| {
        menu.actions.push(action.clone());
    })
    .with_children(|p| text(p, label, 16.0, Color::srgb(0.9, 0.95, 0.95)));
}
fn field(p: &mut ChildSpawnerCommands, menu: &Frontend, label: &str, f: Field, value: &str) {
    button(
        p,
        format!(
            "{label}: {}{}",
            if value.is_empty() {
                "click to enter"
            } else {
                value
            },
            if menu.focus == Some(f) { "  |" } else { "" }
        ),
        MenuAction::Field(f),
    );
}
#[allow(clippy::too_many_arguments)]
pub fn draw(
    mut commands: Commands,
    mut menu: ResMut<Frontend>,
    prefs: Res<Preferences>,
    voice: Res<Voice>,
    chat: Res<ChatInput>,
    mut game: ResMut<Session>,
    mut previous_voice: Local<String>,
    progress: Res<crate::progression::Progress>,
) {
    if !menu.visible {
        return;
    }
    let audio_key = format!(
        "{:?}",
        (
            voice.volume,
            voice.gate_db,
            voice.listen_seconds,
            voice.input,
            voice.output,
            voice.enabled,
            &voice.inputs,
            &voice.outputs,
            &voice.status
        )
    );
    if *previous_voice != audio_key {
        menu.dirty = true;
        *previous_voice = audio_key;
    }
    if !menu.dirty {
        return;
    }
    menu.dirty = false;
    game.dirty = false;
    let white = Color::srgb(0.9, 0.95, 0.95);
    let muted = Color::srgb(0.55, 0.7, 0.73);
    commands.spawn((ui::UiRoot, ui::PendingUi, Visibility::Hidden, Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), padding: UiRect::all(px(32)), flex_direction: FlexDirection::Column, row_gap: px(16), ..default() }, BackgroundColor(Color::srgb(0.025,0.045,0.065))))
    .with_children(|p| {
        text(p, "WRONG NUMBER", 42.0, white);
        text(p, "DISPATCH OPERATIONS  /  EVERY CALL IS A DECISION", 14.0, muted);
        p.spawn(Node { flex_grow: 1.0, min_height: px(0), column_gap: px(28), ..default() }).with_children(|p| {
            p.spawn(Node { width: percent(if menu.page == Page::Settings { 50.0 } else { 52.0 }), flex_direction: FlexDirection::Column, overflow: Overflow::scroll_y(), min_height: px(0), row_gap: px(7), padding: UiRect::all(px(16)), ..default() }).with_children(|p| {
                match menu.page {
                    Page::Main => {
                        field(p, &menu, "Dispatcher name", Field::Name, &menu.name);
                        text(p, "YOUR NEXT SHIFT", 26.0, white);
                        button(p, "Host a room", MenuAction::Page(Page::Host));
                        button(p, "Join a room", MenuAction::Page(Page::Join));
                        button(p, "Join with code", MenuAction::Page(Page::Code));
                        button(p, "Solo shift", MenuAction::Solo);
                        button(p, "Settings", MenuAction::Settings);
                        button(p, "Quit", MenuAction::Quit);
                    }
                    Page::Host => {
                        text(p, "HOST A ROOM", 26.0, white);
                        if !std::env::args().any(|a| a == "--caller-service") {
                            text(p, "AI caller service is not connected. You can host a room, but live calls need the AI launcher and host credentials.", 16.0, Color::srgb(0.95,0.68,0.3));
                        }
                        field(p, &menu, "Dispatcher name", Field::Name, &menu.name);
                        field(p, &menu, "TCP port", Field::Port, &menu.port);
                        field(p, &menu, "Your public IPv4 (or LAN IPv4)", Field::Advertise, &menu.advertise);
                        text(p, "Your computer runs the room for up to six players. For internet play, forward this TCP port to your computer and allow it through the firewall. Use your router's WAN address. Carrier-grade NAT needs a public address from your ISP or a shared VPN.", 16.0, muted);
                        if menu.pending.is_none() { button(p, "Create room & join code", MenuAction::Host); }
                    }
                    Page::Join | Page::Code => {
                        text(p, if menu.page == Page::Code { "JOIN WITH CODE" } else { "JOIN A ROOM" }, 26.0, white);
                        field(p, &menu, "Dispatcher name", Field::Name, &menu.name);
                        if menu.page == Page::Code { field(p, &menu, "Join code", Field::Code, &menu.code); }
                        else { field(p, &menu, "Host IP:port", Field::Address, &menu.address); button(p, "Use a join code instead", MenuAction::Page(Page::Code)); }
                        text(p, "Ask your host for their code or address. Ctrl+V pastes; Ctrl+A clears the focused field. The host must keep the game open.", 16.0, muted);
                        if menu.pending.is_none() { button(p, "Connect", MenuAction::Join); }
                    }
                    Page::Settings => {
                        text(p, "CONTROLS & DISPLAY", 24.0, white);
                        text(p, format!("Mouse sensitivity: {:.1}×", prefs.sensitivity), 16.0, muted);
                        button(p, "Sensitivity −", MenuAction::Sensitivity(false)); button(p, "Sensitivity +", MenuAction::Sensitivity(true));
                        button(p, format!("Invert mouse Y: {}", on_off(prefs.invert_y)), MenuAction::Invert);
                        for (i,label) in LABELS.iter().enumerate() { button(p, format!("{label}: {}{}", key_label(&prefs.keys[i]), if menu.rebind == Some(i) { " — press key" } else { "" }), MenuAction::Bind(i)); }
                        text(p, "Mouse aims • left click interacts • Esc opens menu • F3 toggles performance", 14.0, muted);
                        button(p, format!("Fullscreen: {}", on_off(prefs.fullscreen)), MenuAction::Fullscreen);
                        button(p, format!("VSync: {}", on_off(prefs.vsync)), MenuAction::Vsync);
                        button(p, format!("Show FPS: {}", on_off(prefs.show_fps)), MenuAction::Fps);
                        text(p, "ACCESSIBILITY", 22.0, white);
                        button(p, format!("Interface size: {:.0}%", prefs.text_scale*100.0), MenuAction::TextScale);
                        button(p, format!("Caller subtitles: {}", on_off(prefs.subtitles)), MenuAction::Subtitles);
                        button(p, format!("High contrast: {}", on_off(prefs.high_contrast)), MenuAction::Contrast);
                        button(p, format!("Reduced motion: {}", on_off(prefs.reduced_motion)), MenuAction::Motion);
                        text(p, "Status labels accompany colors. Reduced motion freezes drifting clouds and avatar walk animation; camera shake is never used.", 14.0, muted);
                        button(p, "Reset settings to defaults", MenuAction::Reset);
                    }
                }

            });
            p.spawn(Node { width: percent(48.0), flex_grow: 1.0, min_width: px(0), min_height: px(0), overflow: Overflow::scroll_y(), padding: UiRect::all(px(16)), flex_direction: FlexDirection::Column, row_gap: px(12), ..default() }).with_children(|p| {
                if menu.page == Page::Settings {
                    text(p, "AUDIO & MICROPHONE", 24.0, white);
                    button(p, format!("Microphone mode: {}", ["Voice activation", "Push to talk", "Muted"][prefs.voice_mode as usize]), MenuAction::Mode);
                    text(p, format!("Hold {} to speak in Push to Talk mode; release to send. Voice activation sends after 2 seconds of silence. Both modes stop recording at 12 seconds maximum. This talks to callers; teammate voice chat is not enabled.", key_label(&prefs.keys[7])), 14.0, muted);
                    text(p, format!("Telephone ring volume: {:.0}%", prefs.ring_volume * 100.0), 16.0, muted);
                    button(p, "Telephone ring volume −", MenuAction::RingVolume(false));
                    button(p, "Telephone ring volume +", MenuAction::RingVolume(true));
                    text(p, format!("City & room ambience: {:.0}%", prefs.ambient_volume*100.0), 16.0, muted);
                    button(p,"Ambience -",MenuAction::AmbientVolume(false));
                    button(p,"Ambience +",MenuAction::AmbientVolume(true));
                    ui::audio_ui(p, &voice, &chat);
                } else {
                    text(p, "ONE ROOM. SIX DISPATCHERS.", 24.0, white);
                    text(p, format!("Career: {} completed shifts", progress.career.completed), 16.0, muted);
                    if progress.career.commendation {text(p, "★ COMMENDED DISPATCHER", 22.0, Color::srgb(1.0,0.78,0.25));}
                    else {text(p, "Earn a B or A review to unlock the permanent Commended Dispatcher profile badge.", 14.0, muted);}
                    text(p, &progress.status, 14.0, muted);
                    text(p, "Solo / Host resumes your saved run at its last shift boundary. New runs reset team funding and upgrades; your cosmetic career badge remains.", 14.0, muted);
                    text(p, "Listen to callers. Find out what is happening. Send the right response.\n\nThe creator is the host and controls when the shared shift starts. Joining players do not need an AI API key.", 18.0, muted);
                    text(p, "Direct internet hosting\nJoin codes contain the host address; they do not provide matchmaking or bypass router settings.", 16.0, muted);
                }
            });
        });
        if menu.page != Page::Main { button(p, "Back", MenuAction::Back); }
        text(p, &menu.status, 16.0, Color::srgb(0.95,0.68,0.3));
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codes_round_trip_and_reject_damage() {
        for s in [
            "203.0.113.25:47000",
            "127.0.0.1:47000",
            "[2001:db8::abcd]:65535",
        ] {
            let a = s.parse().unwrap();
            let code = encode_code(a);
            assert_eq!(decode_code(&code.to_lowercase()).unwrap(), a);
            let mut damaged = code.clone();
            damaged.pop();
            damaged.push(if code.ends_with('0') { '1' } else { '0' });
            assert!(decode_code(&damaged).is_err());
        }
        for s in ["", "WN1-TEST", "WN1-😀😀😀😀", "fake"] {
            assert!(decode_code(s).is_err());
        }
        assert!(decode_code(&encode_code("0.0.0.0:47000".parse().unwrap())).is_err());
    }
    #[test]
    fn bindings_and_settings_round_trip() {
        let mut p = Preferences {
            sensitivity: 2.0,
            voice_mode: 1,
            ..default()
        };
        p.keys[7] = "KeyB".into();
        let restored: Preferences =
            serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        let mut input = ButtonInput::default();
        input.press(KeyCode::KeyB);
        assert!(restored.pressed(&input, 7));
        assert!(restored.held(&input, 7));
        assert!(!restored.held(&input, 0));
        assert_eq!(restored.voice_mode, 1);
        assert_eq!(restored.sensitivity, 2.0);
    }
}

/// Rendered acceptance path, driving the same observers as real clicks.
#[allow(clippy::too_many_arguments)]
pub fn verify(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut ready: ResMut<crate::world::ReadyInput>,
    mut menu: ResMut<Frontend>,
    prefs: Res<Preferences>,
    voice: Res<Voice>,
    net: Res<Network>,
    buttons: Query<(Entity, &MenuButton)>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    if !std::env::args().any(|a| a == "--menu-test") {
        return;
    }
    *frame += 1;
    let click = |commands: &mut Commands, action: MenuAction| {
        let (entity, _) = buttons
            .iter()
            .find(|(_, b)| b.0 == action)
            .unwrap_or_else(|| panic!("Missing menu button {action:?}"));
        commands.trigger(Activate { entity });
    };
    let capture = |commands: &mut Commands, name: &str| {
        let args: Vec<_> = std::env::args().collect();
        if let Some(dir) = args
            .iter()
            .position(|a| a == "--capture-dir")
            .and_then(|i| args.get(i + 1))
        {
            std::fs::create_dir_all(dir).unwrap();
            commands
                .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(format!(
                    "{dir}/{name}.png"
                )));
        }
    };
    match *frame {
        90 => {
            assert!(menu.visible);
            capture(&mut commands, "main-menu");
        }
        100 => click(&mut commands, MenuAction::Settings),
        115 => click(&mut commands, MenuAction::Mode),
        125 => {
            assert!(voice.push_to_talk);
            assert_eq!(prefs.voice_mode, 1);
            capture(&mut commands, "settings");
        }
        128 => click(&mut commands, MenuAction::TextScale),
        130 => click(&mut commands, MenuAction::Contrast),
        132 => click(&mut commands, MenuAction::Motion),
        134 => click(&mut commands, MenuAction::Subtitles),
        135 => click(&mut commands, MenuAction::Sensitivity(true)),
        145 => {
            assert!(prefs.sensitivity > 1.0);
            let saved = Preferences::load();
            assert_eq!(saved.voice_mode, 1);
            assert!(
                saved.text_scale > 1.0
                    && saved.high_contrast
                    && saved.reduced_motion
                    && !saved.subtitles
            );
            capture(&mut commands, "accessibility");
            click(&mut commands, MenuAction::Back);
        }
        155 => click(&mut commands, MenuAction::Page(Page::Code)),
        165 => {
            menu.code = "bad-code".into();
            menu.dirty = true;
            click(&mut commands, MenuAction::Join);
        }
        175 => {
            assert!(menu.visible && menu.status.contains("WN1"));
            capture(&mut commands, "join-code");
        }
        185 => click(&mut commands, MenuAction::Back),
        195 => click(&mut commands, MenuAction::Page(Page::Host)),
        205 => {
            menu.advertise = "127.0.0.1".into();
            menu.port = "47991".into();
            menu.dirty = true;
        }
        215 => capture(&mut commands, "host-menu"),
        225 => click(&mut commands, MenuAction::Host),
        260..=275 => {
            ready.active = true;
            ready.elapsed = 0.9;
            if *frame == 270 {
                capture(&mut commands, "door-hold-progress");
            }
        }
        276 => {
            *ready = default();
        }
        280 => {
            menu.actions.push(MenuAction::CopyCode);
        }
        300 => {
            assert!(!menu.visible && net.id == Some(1) && net.online());
            assert_eq!(decode_code(&menu.hosted_code).unwrap().port(), 47991);
            capture(&mut commands, "host-lobby");
        }
        310 => menu.actions.push(MenuAction::Leave),
        340 => {
            assert!(menu.visible && !net.online());
            println!(
                "MENU PASS: settings, push-to-talk mode, invalid code, host authority, share code and leave"
            );
            exit.write(bevy::app::AppExit::Success);
        }
        _ => {}
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;
    #[test]
    fn join_code_connects_to_creator_and_host_departure_disconnects() {
        let mut host = Network::from_options(vec![
            "--host".into(),
            "127.0.0.1:0".into(),
            "--name".into(),
            "Creator".into(),
        ])
        .unwrap();
        let mut host_game = host.initial_game();
        let address: SocketAddr = host.address.parse().unwrap();
        let code = encode_code(address);
        let resume =
            std::env::temp_dir().join(format!("wrong-number-menu-{}.json", uuid::Uuid::new_v4()));
        let mut client = Network::from_options(vec![
            "--join".into(),
            decode_code(&code).unwrap().to_string(),
            "--name".into(),
            "Friend".into(),
            "--resume".into(),
            resume.display().to_string(),
        ])
        .unwrap();
        let mut game = client.initial_game();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while client.id.is_none() && std::time::Instant::now() < deadline {
            host.poll(&mut host_game);
            host.publish(&mut host_game, 0.25, crate::room::Pose::spawn(1));
            client.poll(&mut game);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(host.id, Some(1));
        assert_eq!(client.id, Some(2));
        assert_eq!(host_game.phase, crate::game::Phase::Lobby);
        drop(host);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while client.id.is_some() && std::time::Instant::now() < deadline {
            client.poll(&mut game);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(client.id.is_none());
        assert!(client.status.contains("Disconnected"));
        let _ = std::fs::remove_file(resume);
    }
}
