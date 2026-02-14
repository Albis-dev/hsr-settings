use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use serde::{Deserialize, Serialize};
use winreg::{enums::*, RegKey, RegValue};

const REG_PATH: &str = r"Software\Cognosphere\Star Rail";
const REG_VALUE: &str = "GraphicsSettings_Model_h2986158309";

// ---------------------------------------------------------------------------
// Localization
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Lang {
    En,
    Ko,
    Ja,
}

struct L10n {
    title: &'static str,
    hint: &'static str,
    saved: &'static str,
    save_failed: &'static str,
    no_registry: &'static str,
    on: &'static str,
    off: &'static str,
    fps: &'static str,
    vsync: &'static str,
    render_scale: &'static str,
    resolution_quality: &'static str,
    shadow_quality: &'static str,
    light_quality: &'static str,
    character_quality: &'static str,
    env_detail: &'static str,
    reflection_quality: &'static str,
    sfx_quality: &'static str,
    bloom_quality: &'static str,
    anti_aliasing: &'static str,
    self_shadow: &'static str,
    dlss_quality: &'static str,
    particle_trail: &'static str,
}

fn l10n(lang: Lang) -> &'static L10n {
    match lang {
        Lang::En => &L10n {
            title: " Star Rail Graphics Settings ",
            hint: " \u{2191}\u{2193} Navigate  \u{2190}\u{2192} Change  S Save  Q Quit ",
            saved: "Settings saved.",
            save_failed: "Save failed",
            no_registry: "Registry key not found \u{2014} using defaults. Save to create it.",
            on: "On",
            off: "Off",
            fps: "FPS",
            vsync: "VSync",
            render_scale: "Render Scale",
            resolution_quality: "Resolution Quality",
            shadow_quality: "Shadow Quality",
            light_quality: "Light Quality",
            character_quality: "Character Quality",
            env_detail: "Environment Detail",
            reflection_quality: "Reflection Quality",
            sfx_quality: "SFX Quality",
            bloom_quality: "Bloom Quality",
            anti_aliasing: "Anti-Aliasing",
            self_shadow: "Self Shadow",
            dlss_quality: "DLSS Quality",
            particle_trail: "Particle Trail",
        },
        Lang::Ko => &L10n {
            title: " 붕괴 : 스타레일 그래픽 설정 ",
            hint: " \u{2191}\u{2193} 이동  \u{2190}\u{2192} 변경  S 저장  Q 종료 ",
            saved: "설정이 저장되었습니다.",
            save_failed: "저장 실패",
            no_registry: "레지스트리 키를 찾을 수 없습니다 \u{2014} 기본값 사용 중. 저장하여 생성하세요.",
            on: "켜기",
            off: "끄기",
            fps: "FPS",
            vsync: "수직 동기화",
            render_scale: "렌더 스케일",
            resolution_quality: "해상도 품질",
            shadow_quality: "그림자 품질",
            light_quality: "조명 품질",
            character_quality: "캐릭터 품질",
            env_detail: "환경 디테일",
            reflection_quality: "반사 품질",
            sfx_quality: "효과 품질",
            bloom_quality: "블룸 품질",
            anti_aliasing: "안티앨리어싱",
            self_shadow: "셀프 쉘도우",
            dlss_quality: "DLSS 품질",
            particle_trail: "파티클 트레일",
        },
        Lang::Ja => &L10n {
            title: " 崩壊：スターレイル グラフィック設定 ",
            hint: " \u{2191}\u{2193} 移動  \u{2190}\u{2192} 変更  S 保存  Q 終了 ",
            saved: "設定が保存されました。",
            save_failed: "保存失敗",
            no_registry: "レジストリキーが見つかりません \u{2014} デフォルト値を使用中。保存して作成してください。",
            on: "オン",
            off: "オフ",
            fps: "FPS",
            vsync: "垂直同期",
            render_scale: "レンダースケール",
            resolution_quality: "解像度品質",
            shadow_quality: "影の品質",
            light_quality: "ライト品質",
            character_quality: "キャラクター品質",
            env_detail: "環境ディテール",
            reflection_quality: "反射品質",
            sfx_quality: "エフェクト品質",
            bloom_quality: "ブルーム品質",
            anti_aliasing: "アンチエイリアス",
            self_shadow: "セルフシャドウ",
            dlss_quality: "DLSS品質",
            particle_trail: "パーティクルトレイル",
        },
    }
}

// ---------------------------------------------------------------------------
// Settings model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GraphicsSettings {
    #[serde(rename = "FPS")]
    fps: i64,
    #[serde(rename = "EnableVSync")]
    enable_vsync: bool,
    render_scale: f64,
    resolution_quality: i64,
    shadow_quality: i64,
    light_quality: i64,
    character_quality: i64,
    env_detail_quality: i64,
    reflection_quality: i64,
    #[serde(rename = "SFXQuality")]
    sfx_quality: i64,
    bloom_quality: i64,
    #[serde(rename = "AAMode")]
    aa_mode: i64,
    #[serde(rename = "EnableMetalFXSU")]
    enable_metal_fxsu: bool,
    enable_half_res_transparent: bool,
    enable_self_shadow: i64,
    dlss_quality: i64,
    particle_trail_smoothness: i64,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            fps: 60,
            enable_vsync: true,
            render_scale: 1.0,
            resolution_quality: 3,
            shadow_quality: 3,
            light_quality: 3,
            character_quality: 3,
            env_detail_quality: 3,
            reflection_quality: 3,
            sfx_quality: 3,
            bloom_quality: 3,
            aa_mode: 1,
            enable_metal_fxsu: false,
            enable_half_res_transparent: false,
            enable_self_shadow: 1,
            dlss_quality: 0,
            particle_trail_smoothness: 3,
        }
    }
}

// ---------------------------------------------------------------------------
// Registry I/O
// ---------------------------------------------------------------------------

fn read_settings() -> (GraphicsSettings, bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey(REG_PATH) else {
        return (GraphicsSettings::default(), false);
    };
    let Ok(val) = key.get_raw_value(REG_VALUE) else {
        return (GraphicsSettings::default(), false);
    };
    let json = String::from_utf8_lossy(&val.bytes)
        .trim_end_matches('\0')
        .to_string();
    match serde_json::from_str::<GraphicsSettings>(&json) {
        Ok(s) => (s, true),
        Err(_) => (GraphicsSettings::default(), false),
    }
}

fn write_settings(settings: &GraphicsSettings) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(REG_PATH)?;
    let mut json = serde_json::to_string(settings)?;
    json.push('\0');
    key.set_raw_value(
        REG_VALUE,
        &RegValue {
            vtype: REG_BINARY,
            bytes: json.into_bytes(),
        },
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Setting field identifiers (no fragile index mapping)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Field {
    Fps,
    VSync,
    RenderScale,
    ResolutionQuality,
    ShadowQuality,
    LightQuality,
    CharacterQuality,
    EnvDetailQuality,
    ReflectionQuality,
    SfxQuality,
    BloomQuality,
    AaMode,
    SelfShadow,
    DlssQuality,
    ParticleTrail,
}

// ---------------------------------------------------------------------------
// TUI setting descriptors
// ---------------------------------------------------------------------------

#[derive(Clone)]
enum SettingKind {
    SelectI64(Vec<(&'static str, i64)>),
    SelectF64(Vec<(&'static str, f64)>),
    Toggle,
}

#[derive(Clone)]
struct SettingDef {
    field: Field,
    kind: SettingKind,
}

impl SettingDef {
    fn label(&self, t: &L10n) -> &'static str {
        match self.field {
            Field::Fps               => t.fps,
            Field::VSync             => t.vsync,
            Field::RenderScale       => t.render_scale,
            Field::ResolutionQuality => t.resolution_quality,
            Field::ShadowQuality     => t.shadow_quality,
            Field::LightQuality      => t.light_quality,
            Field::CharacterQuality  => t.character_quality,
            Field::EnvDetailQuality  => t.env_detail,
            Field::ReflectionQuality => t.reflection_quality,
            Field::SfxQuality        => t.sfx_quality,
            Field::BloomQuality      => t.bloom_quality,
            Field::AaMode            => t.anti_aliasing,
            Field::SelfShadow        => t.self_shadow,
            Field::DlssQuality       => t.dlss_quality,
            Field::ParticleTrail     => t.particle_trail,
        }
    }
}

fn setting_defs() -> Vec<SettingDef> {
    let quality: Vec<(&str, i64)> = (1..=5).map(|i| (leak_str(i.to_string()), i)).collect();

    vec![
        SettingDef { field: Field::Fps,               kind: SettingKind::SelectI64(vec![("30", 30), ("60", 60), ("120", 120)]) },
        SettingDef { field: Field::VSync,             kind: SettingKind::Toggle },
        SettingDef { field: Field::RenderScale,       kind: SettingKind::SelectF64(
            (6..=20).step_by(2).map(|v| { let f = v as f64 / 10.0; (leak_str(format!("{f:.1}")), f) }).collect(),
        )},
        SettingDef { field: Field::ResolutionQuality, kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::ShadowQuality,     kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::LightQuality,      kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::CharacterQuality,  kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::EnvDetailQuality,  kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::ReflectionQuality, kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::SfxQuality,        kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::BloomQuality,      kind: SettingKind::SelectI64(quality.clone()) },
        SettingDef { field: Field::AaMode,            kind: SettingKind::SelectI64(vec![("Off", 0), ("On", 1)]) },
        SettingDef { field: Field::SelfShadow,        kind: SettingKind::SelectI64(vec![("Off", 0), ("On", 1)]) },
        SettingDef { field: Field::DlssQuality,       kind: SettingKind::SelectI64(
            std::iter::once(("Off", 0i64)).chain((1..=5).map(|i| (leak_str(i.to_string()), i))).collect(),
        )},
        SettingDef { field: Field::ParticleTrail,     kind: SettingKind::SelectI64(quality) },
    ]
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

// ---------------------------------------------------------------------------
// Field accessors
// ---------------------------------------------------------------------------

fn get_i64(s: &GraphicsSettings, f: Field) -> i64 {
    match f {
        Field::Fps               => s.fps,
        Field::ResolutionQuality => s.resolution_quality,
        Field::ShadowQuality     => s.shadow_quality,
        Field::LightQuality      => s.light_quality,
        Field::CharacterQuality  => s.character_quality,
        Field::EnvDetailQuality  => s.env_detail_quality,
        Field::ReflectionQuality => s.reflection_quality,
        Field::SfxQuality        => s.sfx_quality,
        Field::BloomQuality      => s.bloom_quality,
        Field::AaMode            => s.aa_mode,
        Field::SelfShadow        => s.enable_self_shadow,
        Field::DlssQuality       => s.dlss_quality,
        Field::ParticleTrail     => s.particle_trail_smoothness,
        _ => 0,
    }
}

fn set_i64(s: &mut GraphicsSettings, f: Field, v: i64) {
    match f {
        Field::Fps               => s.fps = v,
        Field::ResolutionQuality => s.resolution_quality = v,
        Field::ShadowQuality     => s.shadow_quality = v,
        Field::LightQuality      => s.light_quality = v,
        Field::CharacterQuality  => s.character_quality = v,
        Field::EnvDetailQuality  => s.env_detail_quality = v,
        Field::ReflectionQuality => s.reflection_quality = v,
        Field::SfxQuality        => s.sfx_quality = v,
        Field::BloomQuality      => s.bloom_quality = v,
        Field::AaMode            => s.aa_mode = v,
        Field::SelfShadow        => s.enable_self_shadow = v,
        Field::DlssQuality       => s.dlss_quality = v,
        Field::ParticleTrail     => s.particle_trail_smoothness = v,
        _ => {}
    }
}

fn get_f64(s: &GraphicsSettings, f: Field) -> f64 {
    match f {
        Field::RenderScale => s.render_scale,
        _ => 0.0,
    }
}

fn set_f64(s: &mut GraphicsSettings, f: Field, v: f64) {
    match f {
        Field::RenderScale => s.render_scale = v,
        _ => {}
    }
}

fn get_bool(s: &GraphicsSettings, f: Field) -> bool {
    match f {
        Field::VSync => s.enable_vsync,
        _ => false,
    }
}

fn set_bool(s: &mut GraphicsSettings, f: Field, v: bool) {
    match f {
        Field::VSync => s.enable_vsync = v,
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct App {
    settings: GraphicsSettings,
    defs: Vec<SettingDef>,
    cursor: usize,
    status: String,
    lang: Lang,
}

impl App {
    fn new(lang: Lang) -> Self {
        let (settings, existed) = read_settings();
        let t = l10n(lang);
        let status = if existed {
            String::new()
        } else {
            t.no_registry.into()
        };
        Self {
            settings,
            defs: setting_defs(),
            cursor: 0,
            status,
            lang,
        }
    }

    fn t(&self) -> &'static L10n {
        l10n(self.lang)
    }

    fn cycle(&mut self, delta: isize) {
        let def = &self.defs[self.cursor];
        let field = def.field;
        match &def.kind {
            SettingKind::SelectI64(opts) => {
                let cur = get_i64(&self.settings, field);
                let pos = opts.iter().position(|(_, v)| *v == cur).unwrap_or(0);
                let next = (pos as isize + delta).rem_euclid(opts.len() as isize) as usize;
                set_i64(&mut self.settings, field, opts[next].1);
            }
            SettingKind::SelectF64(opts) => {
                let cur = get_f64(&self.settings, field);
                let pos = opts
                    .iter()
                    .position(|(_, v)| (*v - cur).abs() < 0.001)
                    .unwrap_or(0);
                let next = (pos as isize + delta).rem_euclid(opts.len() as isize) as usize;
                set_f64(&mut self.settings, field, opts[next].1);
            }
            SettingKind::Toggle => {
                let cur = get_bool(&self.settings, field);
                set_bool(&mut self.settings, field, !cur);
            }
        }
    }

    fn save(&mut self) {
        let t = self.t();
        match write_settings(&self.settings) {
            Ok(()) => self.status = t.saved.into(),
            Err(e) => self.status = format!("{}: {e}", t.save_failed),
        }
    }

    fn value_display(&self, idx: usize) -> String {
        let def = &self.defs[idx];
        let t = self.t();
        match &def.kind {
            SettingKind::SelectI64(opts) => {
                let cur = get_i64(&self.settings, def.field);
                opts.iter()
                    .find(|(_, v)| *v == cur)
                    .map(|(l, _)| l.to_string())
                    .unwrap_or_else(|| cur.to_string())
            }
            SettingKind::SelectF64(opts) => {
                let cur = get_f64(&self.settings, def.field);
                opts.iter()
                    .find(|(_, v)| (*v - cur).abs() < 0.001)
                    .map(|(l, _)| l.to_string())
                    .unwrap_or_else(|| format!("{cur:.1}"))
            }
            SettingKind::Toggle => {
                if get_bool(&self.settings, def.field) {
                    t.on.into()
                } else {
                    t.off.into()
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Language picker
// ---------------------------------------------------------------------------

fn draw_lang_picker(frame: &mut Frame, cursor: usize) {
    let area = frame.area();
    let [_, center, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(9),
        Constraint::Fill(1),
    ])
    .areas(area);
    let [_, box_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(34),
        Constraint::Fill(1),
    ])
    .areas(center);

    let options = [
        ("1", "English"),
        ("2", "한국어 (Korean)"),
        ("3", "日本語 (Japanese)"),
    ];

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "  Select Language / 언어 선택 / 言語選択",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, (key, label)) in options.iter().enumerate() {
        let selected = i == cursor;
        let pointer = if selected { "\u{25b8} " } else { "  " };
        let style = if selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(vec![
            Span::styled(pointer, style),
            Span::styled(format!("[{key}] {label}"), style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Enter to confirm",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default().borders(Borders::ALL);
    frame.render_widget(Paragraph::new(lines).block(block), box_area);
}

fn pick_language(terminal: &mut ratatui::DefaultTerminal) -> io::Result<Option<Lang>> {
    let langs = [Lang::En, Lang::Ko, Lang::Ja];
    let mut cursor: usize = 0;

    loop {
        terminal.draw(|f| draw_lang_picker(f, cursor))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                KeyCode::Up | KeyCode::Char('k') => {
                    if cursor > 0 { cursor -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if cursor < langs.len() - 1 { cursor += 1; }
                }
                KeyCode::Char('1') => return Ok(Some(Lang::En)),
                KeyCode::Char('2') => return Ok(Some(Lang::Ko)),
                KeyCode::Char('3') => return Ok(Some(Lang::Ja)),
                KeyCode::Enter => return Ok(Some(langs[cursor])),
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Settings TUI rendering
// ---------------------------------------------------------------------------

fn draw_settings(frame: &mut Frame, app: &App) {
    let t = app.t();

    let [header_area, list_area, status_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    // Header
    let header = Paragraph::new(Line::from(vec![Span::styled(
        t.title,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, header_area);

    // Settings list
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .title(t.hint);
    let inner = inner_block.inner(list_area);
    frame.render_widget(inner_block, list_area);

    let visible_height = inner.height as usize;
    let total = app.defs.len();

    let scroll_offset = if app.cursor >= visible_height {
        app.cursor - visible_height + 1
    } else {
        0
    };

    let lines: Vec<Line> = app
        .defs
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, def)| {
            let selected = i == app.cursor;
            let pointer = if selected { "\u{25b8} " } else { "  " };
            let label = format!("{:<24}", def.label(t));
            let value = format!("  \u{25c2} {} \u{25b8}", app.value_display(i));

            let style = if selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let val_style = if selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Line::from(vec![
                Span::styled(pointer, style),
                Span::styled(label, style),
                Span::styled(value, val_style),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);

    if total > visible_height {
        let mut sb_state = ScrollbarState::new(total).position(scroll_offset);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            list_area,
            &mut sb_state,
        );
    }

    // Status bar
    let status_style = if app.status.contains(t.saved) {
        Style::default().fg(Color::Green)
    } else if !app.status.is_empty() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let status = Paragraph::new(Span::styled(format!(" {}", app.status), status_style))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, status_area);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let lang = match pick_language(&mut terminal)? {
        Some(l) => l,
        None => {
            ratatui::restore();
            return Ok(());
        }
    };

    let mut app = App::new(lang);

    loop {
        terminal.draw(|f| draw_settings(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.cursor > 0 {
                        app.cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.cursor + 1 < app.defs.len() {
                        app.cursor += 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => app.cycle(1),
                KeyCode::Left | KeyCode::Char('h') => app.cycle(-1),
                KeyCode::Char('s') => app.save(),
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(())
}
