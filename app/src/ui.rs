//! Token desain, font, ikon garis, dan widget bersama.
//! Semua warna dan ukuran ada di sini; layar lain hanya memakai token.
use egui::{
    vec2, Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Frame, Margin, Pos2, Rect, Response, RichText, Sense,
    Stroke, StrokeKind, Ui, Vec2,
};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub struct Pal {
    pub dark: bool,
    pub paper: Color32, pub surface: Color32, pub surface2: Color32,
    pub ink: Color32, pub ink2: Color32, pub ink3: Color32, pub line: Color32, pub line2: Color32,
    pub blue: Color32, pub blue_press: Color32, pub blue_soft: Color32, pub blue_ink: Color32, pub on_blue: Color32,
    pub red: Color32, pub red_press: Color32, pub red_soft: Color32, pub red_ink: Color32,
    pub sun: Color32, pub sun_press: Color32, pub sun_soft: Color32, pub sun_ink: Color32,
    pub teal: Color32, pub teal_soft: Color32,
    pub tier: [Color32; 7], pub tier_soft: [Color32; 7], pub tier_ink: [Color32; 7],
}
const fn hex(c: u32) -> Color32 { Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8) }

pub const LIGHT: Pal = Pal {
    dark: false,
    paper: hex(0xFFF9F0), surface: hex(0xFFFFFF), surface2: hex(0xFFF2DF),
    ink: hex(0x1B1E2B), ink2: hex(0x585F73), ink3: hex(0x8A8FA0), line: hex(0xEEE2CF), line2: hex(0xDCCBB0),
    blue: hex(0x2B54F5), blue_press: hex(0x1A39C2), blue_soft: hex(0xE6ECFF), blue_ink: hex(0x1A39C2), on_blue: hex(0xFFFFFF),
    red: hex(0xF23A48), red_press: hex(0xB4141F), red_soft: hex(0xFFE8EA), red_ink: hex(0xB4141F),
    sun: hex(0xFFC23A), sun_press: hex(0xD99500), sun_soft: hex(0xFFF0C7), sun_ink: hex(0x7A4E00),
    teal: hex(0x0F9E8A), teal_soft: hex(0xD8F4EE),
    tier: [hex(0xFFB020), hex(0x2B54F5), hex(0xFF6B4A), hex(0x0F9E8A), hex(0xE8820C), hex(0xD93F78), hex(0x1B2A6B)],
    tier_soft: [hex(0xFFF0C7), hex(0xE6ECFF), hex(0xFFE6DF), hex(0xD8F4EE), hex(0xFDEBD3), hex(0xFBE1EB), hex(0xE1E5F5)],
    tier_ink: [hex(0x7A4E00), hex(0x1A39C2), hex(0xA8321A), hex(0x0A6B5D), hex(0x8A4A00), hex(0x9A1E4E), hex(0x1B2A6B)],
};
pub const DARK: Pal = Pal {
    dark: true,
    paper: hex(0x12141C), surface: hex(0x1B1E29), surface2: hex(0x232737),
    ink: hex(0xF3F1EA), ink2: hex(0xAEB3C2), ink3: hex(0x7D8294), line: hex(0x2C3142), line2: hex(0x3B4156),
    blue: hex(0x7C95FF), blue_press: hex(0x5373F0), blue_soft: hex(0x222B50), blue_ink: hex(0xAFBEFF), on_blue: hex(0x0B1230),
    red: hex(0xFF6B76), red_press: hex(0xD9414D), red_soft: hex(0x3B1D24), red_ink: hex(0xFFA3AA),
    sun: hex(0xFFC94D), sun_press: hex(0xD9A21E), sun_soft: hex(0x3A3018), sun_ink: hex(0xFFD877),
    teal: hex(0x3CCDB7), teal_soft: hex(0x15342F),
    tier: [hex(0xFFB020), hex(0x7C95FF), hex(0xFF6B4A), hex(0x3CCDB7), hex(0xE8820C), hex(0xD93F78), hex(0x8E9BD6)],
    tier_soft: [hex(0x3A3018), hex(0x222B50), hex(0x3D231C), hex(0x15342F), hex(0x3A2A14), hex(0x3A1B28), hex(0x232846)],
    tier_ink: [hex(0xFFD877), hex(0xAFBEFF), hex(0xFFB09C), hex(0x7FE3D3), hex(0xFFC27A), hex(0xFF9DC0), hex(0xC3CCF5)],
};

thread_local! { static PAL: Cell<Pal> = const { Cell::new(LIGHT) }; }
pub fn p() -> Pal { PAL.with(|c| c.get()) }
pub fn set_pal(pl: Pal) { PAL.with(|c| c.set(pl)); }

pub const R: u8 = 8; // sudut paling besar
pub const PRESS: f32 = 3.0;

/* ── Font ── */
pub fn ui_font(size: f32) -> FontId { FontId::new(size, FontFamily::Proportional) }
pub fn semi(size: f32) -> FontId { FontId::new(size, FontFamily::Name("semi".into())) }
pub fn bold(size: f32) -> FontId { FontId::new(size, FontFamily::Name("bold".into())) }
pub fn display(size: f32) -> FontId { FontId::new(size, FontFamily::Name("display".into())) }
pub fn serif(size: f32) -> FontId { FontId::new(size, FontFamily::Name("serif".into())) }

pub fn install_fonts(ctx: &egui::Context) {
    let mut defs = egui::FontDefinitions::default();
    let dir = std::path::Path::new(&std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into())).join("Fonts");
    let mut load = |key: &str, files: &[&str]| -> bool {
        for f in files {
            if let Ok(b) = std::fs::read(dir.join(f)) {
                defs.font_data.insert(key.to_string(), std::sync::Arc::new(egui::FontData::from_owned(b)));
                return true;
            }
        }
        false
    };
    let has_ui = load("ui", &["segoeui.ttf"]);
    let has_semi = load("semi", &["seguisb.ttf", "segoeuib.ttf"]);
    let has_bold = load("bold", &["segoeuib.ttf"]);
    let has_disp = load("display", &["seguibl.ttf", "segoeuib.ttf"]);
    let has_serif = load("serif", &["georgia.ttf", "cambria.ttc", "times.ttf"]);
    let base: Vec<String> = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let fam = |first: Option<&str>| -> Vec<String> {
        let mut v: Vec<String> = first.into_iter().map(String::from).collect();
        if has_ui && first != Some("ui") { v.push("ui".into()); }
        v.extend(base.iter().cloned());
        v
    };
    defs.families.insert(FontFamily::Proportional, fam(has_ui.then_some("ui")));
    defs.families.insert(FontFamily::Name("semi".into()), fam(has_semi.then_some("semi")));
    defs.families.insert(FontFamily::Name("bold".into()), fam(has_bold.then_some("bold")));
    defs.families.insert(FontFamily::Name("display".into()), fam(has_disp.then_some("display")));
    defs.families.insert(FontFamily::Name("serif".into()), fam(has_serif.then_some("serif")));
    ctx.set_fonts(defs);
}

pub fn apply_visuals(ctx: &egui::Context, pl: &Pal) {
    let mut v = if pl.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = pl.paper;
    v.window_fill = pl.surface;
    v.extreme_bg_color = pl.surface;
    v.faint_bg_color = pl.surface2;
    v.override_text_color = Some(pl.ink);
    v.hyperlink_color = pl.blue_ink;
    v.selection.bg_fill = pl.blue_soft;
    v.selection.stroke = Stroke::new(1.5, pl.blue);
    v.window_corner_radius = CornerRadius::same(R);
    v.window_stroke = Stroke::new(1.0, pl.line2);
    v.window_shadow = egui::Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(if pl.dark { 110 } else { 36 }) };
    v.popup_shadow = v.window_shadow;
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.noninteractive, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(6);
    }
    // bg_fill dipakai rel slider dan latar kotak centang: harus terlihat di atas kartu putih.
    v.widgets.inactive.bg_fill = pl.line2; v.widgets.inactive.weak_bg_fill = pl.surface;
    v.slider_trailing_fill = true;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, pl.line2);
    v.widgets.hovered.bg_fill = pl.surface2; v.widgets.hovered.weak_bg_fill = pl.surface2;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, pl.line2);
    v.widgets.active.bg_fill = pl.blue_soft; v.widgets.active.weak_bg_fill = pl.blue_soft;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, pl.line);
    v.text_cursor.stroke = Stroke::new(2.0, pl.blue);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = vec2(8.0, 8.0);
        s.spacing.button_padding = vec2(12.0, 6.0);
        s.spacing.scroll.bar_width = 8.0;
        s.spacing.slider_width = 200.0;
        s.spacing.interact_size.y = 26.0;
        s.text_styles.insert(egui::TextStyle::Body, ui_font(15.0));
        s.text_styles.insert(egui::TextStyle::Button, semi(15.0));
        s.text_styles.insert(egui::TextStyle::Small, ui_font(12.5));
        s.text_styles.insert(egui::TextStyle::Heading, display(26.0));
    });
}

/* ── Ikon garis 1,5 px (SVG putih, diwarnai lewat tint) ── */
pub fn icon_path(name: &str) -> &'static str {
    match name {
        "home" => "M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z",
        "path" => "M6 21a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM18 7a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM8 19h7.5a3.5 3.5 0 0 0 0-7h-7a3.5 3.5 0 0 1 0-7H16",
        "mic" => "M12 3a3 3 0 0 1 3 3v6a3 3 0 0 1-6 0V6a3 3 0 0 1 3-3zM5 11a7 7 0 0 0 14 0M12 18v3M8.5 21h7",
        "book" => "M5 4.5A1.5 1.5 0 0 1 6.5 3H19v15H6.5A1.5 1.5 0 0 0 5 19.5zM5 19.5A1.5 1.5 0 0 0 6.5 21H19v-3M9 7.5h6",
        "chart" => "M4 20h16M7 16.5v-5M12 16.5V6M17 16.5v-8",
        "sliders" => "M4 7h9m4 0h3M4 17h3m4 0h9M15 4.5v5M9 14.5v5",
        "speaker" => "M4 9.5h3.5L12 6v12l-4.5-3.5H4zM15.5 9.5a3.5 3.5 0 0 1 0 5M18 7a7 7 0 0 1 0 10",
        "slow" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 7.5V12l3 2",
        "check" => "M5 12.5l4.5 4.5L19 7.5",
        "x" => "M6.5 6.5l11 11M17.5 6.5l-11 11",
        "flame" => "M12 21c-3.9 0-6.5-2.6-6.5-6.2 0-2.9 1.9-4.8 3.3-6.3.3 1.9 1.4 2.9 2.4 2.9 0-2.9 1-5.8 3.8-7.9 0 3.8 4.5 5.8 4.5 11.1C19.5 18.4 16.5 21 12 21z",
        "lock" => "M6 11h12v9H6zM8.5 11V8a3.5 3.5 0 0 1 7 0v3",
        "star" => "M12 3.5l2.6 5.3 5.9.9-4.3 4.1 1 5.8L12 16.8l-5.2 2.8 1-5.8-4.3-4.1 5.9-.9z",
        "search" => "M10.5 17.5a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-4.5-4.5",
        "play" => "M8 5.5v13l10.5-6.5z",
        "stop" => "M7.5 7.5h9v9h-9z",
        "help" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM9.6 9.4a2.5 2.5 0 1 1 3.4 2.4c-.6.3-1 .8-1 1.5v.7M12 17h.01",
        "right" => "M5 12h14M13 6l6 6-6 6",
        "chat" => "M4 5h16v11H9.5L4 20z",
        "repeat" => "M4.5 11.5A7.5 7.5 0 0 1 18 7l1.5 1.5M19.5 4v4.5H15M19.5 12.5A7.5 7.5 0 0 1 6 17l-1.5-1.5M4.5 20v-4.5H9",
        "bookmark" => "M6.5 3.5h11v17l-5.5-4-5.5 4z",
        "trophy" => "M8 4h8v5a4 4 0 0 1-8 0zM8 6H5a3 3 0 0 0 3 4M16 6h3a3 3 0 0 1-3 4M12 13v4M8.5 21h7M9.5 17h5v4h-5z",
        "note" => "M6 3h9l4 4v14H6zM14.5 3v4.5H19M9 12.5h7M9 16.5h5",
        "ear" => "M4 15v-3a8 8 0 0 1 16 0v3M4 15h3v5H4zM17 15h3v5h-3z",
        "keyboard" => "M3 7h18v10H3zM7 10.5h.01M10.5 10.5h.01M14 10.5h.01M17.5 10.5h.01M8 14h8",
        "trash" => "M4 7h16M9.5 7V4.5h5V7M6 7l1 13h10l1-13M10 11v5M14 11v5",
        "download" => "M12 4v11M7 10l5 5 5-5M5 20h14",
        "upload" => "M12 20V9M7 14l5-5 5 5M5 4h14",
        "target" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM12 12h.01",
        "plane" => "M3 11.5 20.5 4l-7.5 17.5-2.2-7.8zM10.8 13.7 20.5 4",
        "briefcase" => "M4 8h16v11H4zM9 8V5h6v3M4 13h16",
        "cap" => "M2.5 9 12 4.5 21.5 9 12 13.5zM6.5 11v5c3 2 8 2 11 0v-5M21.5 9v5",
        "heart" => "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
        "seed" => "M12 21v-8M12 13c0-4 3-7 8-7 0 4-3 7-8 7zM12 15c0-3-2.5-5.5-7-5.5 0 3 2.5 5.5 7 5.5z",
        "bolt" => "M13 3 5 13.5h6L10 21l8-10.5h-6z",
        "bridge" => "M2.5 17.5h19M5 17.5c0-5 3.1-8.5 7-8.5s7 3.5 7 8.5M9 17.5v-4.8M15 17.5v-4.8M12 17.5V9",
        _ => "",
    }
}
static ICONS: OnceLock<std::sync::Mutex<HashMap<String, &'static [u8]>>> = OnceLock::new();
fn icon_bytes(name: &str) -> &'static [u8] {
    let m = ICONS.get_or_init(Default::default);
    let mut g = m.lock().unwrap();
    g.entry(name.to_string()).or_insert_with(|| {
        let fill = if name == "play" || name == "stop" { "#fff" } else { "none" };
        let svg = format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="96" height="96"><path d="{}" fill="{fill}" stroke="#fff" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>"##, icon_path(name));
        Box::leak(svg.into_bytes().into_boxed_slice())
    })
}
pub fn icon_at(ui: &Ui, name: &str, rect: Rect, color: Color32) {
    egui::Image::from_bytes(format!("bytes://ic-{name}.svg"), icon_bytes(name)).tint(color).paint_at(ui, rect);
}
pub fn icon(ui: &mut Ui, name: &str, size: f32, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    icon_at(ui, name, rect, color);
    resp
}

/* ── Teks ── */
pub fn txt(s: impl Into<String>, size: f32) -> RichText { RichText::new(s).font(ui_font(size)) }
/// Teks petunjuk di kolom ketik: warna redup agar tidak terbaca seperti isian.
pub fn hint(s: &str) -> RichText { RichText::new(s).color(p().ink3) }
pub fn h1(ui: &mut Ui, s: &str) { ui.label(RichText::new(s).font(display(32.0)).color(p().ink)); }
pub fn h2(ui: &mut Ui, s: &str) { ui.label(RichText::new(s).font(display(24.0)).color(p().ink)); }
pub fn h3(ui: &mut Ui, s: &str) { ui.label(RichText::new(s).font(bold(17.0)).color(p().ink)); }
pub fn muted(ui: &mut Ui, s: &str) { ui.label(RichText::new(s).font(ui_font(14.0)).color(p().ink2)); }
pub fn small(ui: &mut Ui, s: &str, c: Color32) { ui.label(RichText::new(s).font(ui_font(12.5)).color(c)); }
pub fn eyebrow(ui: &mut Ui, s: &str, c: Color32) { ui.label(RichText::new(s.to_uppercase()).font(bold(11.5)).color(c).extra_letter_spacing(0.8)); }
pub fn fr_label(ui: &mut Ui, s: &str, size: f32) -> Response { ui.label(RichText::new(s).font(serif(size)).color(p().ink)) }

/// Seperti `ui.columns`, tetapi isi tiap kolom rata kiri (kolom bawaan egui meratakan teks kiri-kanan).
pub fn cols<R>(ui: &mut Ui, n: usize, add: impl FnOnce(&mut [Ui]) -> R) -> R {
    ui.columns(n, |c| {
        let mut kids: Vec<Ui> = c.iter_mut()
            .map(|u| u.new_child(egui::UiBuilder::new().max_rect(u.max_rect()).layout(egui::Layout::top_down(egui::Align::Min))))
            .collect();
        let r = add(&mut kids);
        for (u, k) in c.iter_mut().zip(kids.iter()) { u.advance_cursor_after_rect(k.min_rect()); }
        r
    })
}

/* ── Bingkai ── */
pub fn card() -> Frame {
    let pl = p();
    Frame::new().fill(pl.surface).stroke(Stroke::new(1.0, pl.line)).corner_radius(CornerRadius::same(R)).inner_margin(Margin::same(20))
}
pub fn soft(fill: Color32) -> Frame { Frame::new().fill(fill).corner_radius(CornerRadius::same(R)).inner_margin(Margin::symmetric(16, 12)) }

/* ── Tombol bertekanan ── */
#[derive(Clone, Copy, PartialEq)]
pub enum Kind { Primary, Sun, Normal, Ghost, Danger }

pub fn btn(ui: &mut Ui, label: &str, ic: Option<&str>, kind: Kind, big: bool, enabled: bool) -> Response {
    let pl = p();
    let (h, fs, pad) = if big { (50.0, 17.0, 22.0) } else { (38.0, 14.5, 14.0) };
    let font = semi(fs);
    let (fill, fg, base, border) = match kind {
        Kind::Primary => (pl.blue, pl.on_blue, pl.blue_press, pl.blue_press),
        Kind::Sun => (pl.sun, hex(0x3A2600), pl.sun_press, pl.sun_press),
        Kind::Normal => (pl.surface, pl.ink, pl.line2, pl.line2),
        Kind::Ghost => (Color32::TRANSPARENT, pl.ink2, Color32::TRANSPARENT, Color32::TRANSPARENT),
        Kind::Danger => (pl.surface, pl.red_ink, pl.red, pl.red),
    };
    let galley = ui.fonts(|f| f.layout_no_wrap(label.to_string(), font, fg));
    let isz = if big { 20.0 } else { 17.0 };
    let w = pad * 2.0 + galley.size().x + if ic.is_some() { isz + 8.0 } else { 0.0 } - if label.is_empty() { 8.0 + pad * 0.6 } else { 0.0 };
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(vec2(w.max(h), h + PRESS), sense);
    let resp = if enabled { resp.on_hover_cursor(CursorIcon::PointingHand) } else { resp };
    let down = enabled && resp.is_pointer_button_down_on();
    let alpha = if enabled { 1.0 } else { 0.45 };
    let top = rect.min + vec2(0.0, if down { PRESS } else { 0.0 });
    let face = Rect::from_min_size(top, vec2(rect.width(), h));
    let pt = ui.painter();
    if kind != Kind::Ghost && !down {
        pt.rect_filled(Rect::from_min_size(rect.min + vec2(0.0, PRESS), vec2(rect.width(), h)), CornerRadius::same(R), base.gamma_multiply(alpha));
    }
    let hov = enabled && resp.hovered();
    let f = if kind == Kind::Ghost && hov { pl.surface2 } else if hov && matches!(kind, Kind::Normal | Kind::Danger) { pl.surface2 } else { fill };
    pt.rect(face, CornerRadius::same(R), f.gamma_multiply(alpha), Stroke::new(1.0, border.gamma_multiply(alpha)), StrokeKind::Inside);
    let mut x = face.center().x - (w - pad * 2.0) / 2.0;
    if let Some(n) = ic {
        icon_at(ui, n, Rect::from_center_size(Pos2::new(x + isz / 2.0, face.center().y), Vec2::splat(isz)), fg.gamma_multiply(alpha));
        x += isz + 8.0;
    }
    if !label.is_empty() { pt.galley(Pos2::new(x, face.center().y - galley.size().y / 2.0), galley, fg.gamma_multiply(alpha)); }
    resp
}
pub fn icon_btn(ui: &mut Ui, ic: &str, tip: &str, kind: Kind) -> Response {
    let pl = p();
    let s = 36.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(s, s), Sense::click());
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(tip);
    let (bg, fg) = match kind {
        Kind::Primary => (pl.blue, pl.on_blue),
        Kind::Danger => (pl.red_soft, pl.red_ink),
        _ => (if resp.hovered() { pl.surface2 } else { Color32::TRANSPARENT }, pl.ink2),
    };
    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);
    icon_at(ui, ic, rect.shrink(9.0), fg);
    resp
}

/// Sakelar nyala/mati.
pub fn switch(ui: &mut Ui, on: &mut bool) -> Response {
    let pl = p();
    let (rect, mut resp) = ui.allocate_exact_size(vec2(48.0, 28.0), Sense::click());
    if resp.clicked() { *on = !*on; resp.mark_changed(); }
    let t = ui.ctx().animate_bool_with_time(resp.id, *on, 0.15);
    let bg = if *on { pl.blue } else { pl.line2 };
    ui.painter().rect_filled(rect, CornerRadius::same(14), bg);
    let x = egui::lerp((rect.left() + 14.0)..=(rect.right() - 14.0), t);
    ui.painter().circle_filled(Pos2::new(x, rect.center().y), 11.0, Color32::WHITE);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

pub fn chip(ui: &mut Ui, s: &str, bg: Color32, fg: Color32) {
    Frame::new().fill(bg).corner_radius(CornerRadius::same(4)).inner_margin(Margin::symmetric(8, 2)).show(ui, |ui| {
        ui.label(RichText::new(s).font(bold(11.5)).color(fg));
    });
}

pub fn bar(ui: &mut Ui, frac: f32, h: f32, color: Color32, id: egui::Id) {
    let pl = p();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
    let f = ui.ctx().animate_value_with_time(id, frac.clamp(0.0, 1.0), 0.4);
    ui.painter().rect_filled(rect, CornerRadius::same((h / 2.0) as u8), pl.surface2);
    if f > 0.001 {
        let r = Rect::from_min_size(rect.min, vec2(rect.width() * f, h));
        ui.painter().rect_filled(r, CornerRadius::same((h / 2.0) as u8), color);
    }
}

pub fn ring(ui: &mut Ui, frac: f32, size: f32, id: egui::Id) {
    let pl = p();
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let f = ui.ctx().animate_value_with_time(id, frac.clamp(0.0, 1.0), 0.7);
    let c = rect.center();
    let w = size * 0.12;
    let r = size / 2.0 - w / 2.0;
    ui.painter().circle_stroke(c, r, Stroke::new(w, pl.surface2));
    if f > 0.0 {
        let n = 64;
        let pts: Vec<Pos2> = (0..=((n as f32 * f) as usize).max(1))
            .map(|i| { let a = -std::f32::consts::FRAC_PI_2 + (i as f32 / n as f32) * std::f32::consts::TAU; c + vec2(a.cos(), a.sin()) * r })
            .collect();
        ui.painter().add(egui::Shape::line(pts, Stroke::new(w, if frac >= 1.0 { pl.blue } else { pl.sun })));
    }
}

/// Tombol pilihan jawaban (soal pilihan ganda, pasangan kata).
#[derive(Clone, Copy, PartialEq)]
pub enum OptState { Normal, Selected, Right, Wrong, Done }
pub fn opt(ui: &mut Ui, key: Option<usize>, label: &str, french: bool, state: OptState, width: f32, enabled: bool) -> Response {
    let pl = p();
    let font = if french { serif(18.0) } else { semi(15.5) };
    let wrap = width - 70.0;
    let galley = ui.fonts(|f| f.layout(label.to_string(), font, Color32::PLACEHOLDER, wrap));
    let h = (galley.size().y + 28.0).max(58.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, h + PRESS), if enabled { Sense::click() } else { Sense::hover() });
    let resp = if enabled { resp.on_hover_cursor(CursorIcon::PointingHand) } else { resp };
    let down = enabled && resp.is_pointer_button_down_on();
    let (border, fill, base) = match state {
        OptState::Selected => (pl.blue, pl.blue_soft, pl.blue),
        OptState::Right => (pl.blue, pl.blue_soft, pl.blue),
        OptState::Wrong => (pl.red, pl.red_soft, pl.red),
        OptState::Done => (pl.line, Color32::TRANSPARENT, Color32::TRANSPARENT),
        OptState::Normal => (pl.line2, if resp.hovered() && enabled { pl.surface2 } else { pl.surface }, pl.line2),
    };
    let face = Rect::from_min_size(rect.min + vec2(0.0, if down { 2.0 } else { 0.0 }), vec2(width, h));
    let pt = ui.painter();
    if !down && base != Color32::TRANSPARENT { pt.rect_filled(face.translate(vec2(0.0, PRESS)), CornerRadius::same(R), base); }
    pt.rect(face, CornerRadius::same(R), fill, Stroke::new(2.0, border), StrokeKind::Inside);
    let mut x = face.min.x + 16.0;
    if let Some(k) = key {
        let kr = Rect::from_min_size(Pos2::new(x, face.center().y - 13.0), Vec2::splat(26.0));
        pt.rect_stroke(kr, CornerRadius::same(4), Stroke::new(1.0, if state == OptState::Selected { pl.blue } else { pl.line2 }), StrokeKind::Inside);
        pt.text(kr.center(), Align2::CENTER_CENTER, k.to_string(), bold(12.0), if state == OptState::Selected { pl.blue_ink } else { pl.ink3 });
        x += 38.0;
    }
    let tint = if state == OptState::Done { pl.ink3 } else { pl.ink };
    pt.galley(Pos2::new(x, face.center().y - galley.size().y / 2.0), galley, tint);
    match state {
        OptState::Right => icon_at(ui, "check", Rect::from_center_size(Pos2::new(face.max.x - 24.0, face.center().y), Vec2::splat(20.0)), pl.blue),
        OptState::Wrong => icon_at(ui, "x", Rect::from_center_size(Pos2::new(face.max.x - 24.0, face.center().y), Vec2::splat(20.0)), pl.red),
        _ => {}
    }
    resp
}

/// Teks Prancis yang setiap katanya bisa diklik. Mengembalikan (kata, posisi) yang diklik.
pub fn fr_words(ui: &mut Ui, text: &str, size: f32, color: Color32) -> Option<(String, Pos2)> {
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
        let mut word = String::new();
        let mut flush = |ui: &mut Ui, w: &mut String, clicked: &mut Option<(String, Pos2)>| {
            if w.is_empty() { return; }
            let r = ui.add(egui::Label::new(RichText::new(w.as_str()).font(serif(size)).color(color)).sense(Sense::click()));
            let r = r.on_hover_cursor(CursorIcon::PointingHand);
            let y = r.rect.bottom() - 2.0;
            let c = if r.hovered() { p().sun_press } else { p().line2 };
            let mut x = r.rect.left();
            while x < r.rect.right() { ui.painter().line_segment([Pos2::new(x, y), Pos2::new((x + 2.0).min(r.rect.right()), y)], Stroke::new(1.5, c)); x += 4.0; }
            if r.clicked() { *clicked = Some((w.clone(), r.rect.center_bottom())); }
            w.clear();
        };
        for ch in text.chars() {
            if ch.is_alphabetic() || ((ch == '\'' || ch == '’' || ch == '-') && !word.is_empty()) { word.push(ch); }
            else {
                flush(ui, &mut word, &mut clicked);
                ui.label(RichText::new(ch.to_string()).font(serif(size)).color(color));
            }
        }
        flush(ui, &mut word, &mut clicked);
    });
    clicked
}

pub fn tier_name(n: u8) -> String { format!("Tier {} · {}", n, crate::content::c().tiers[n as usize].label) }
pub fn fmt_min(sec: f64) -> String { let m = (sec / 60.0).round() as i64; if m < 60 { format!("{m} mnt") } else { format!("{} j {} mnt", m / 60, m % 60) } }

/// Logo: lengkung jembatan putih di kotak biru dengan titik matahari.
pub const LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="256" height="256"><rect width="24" height="24" rx="5.5" fill="#2B54F5"/><path d="M4 17h16M6.5 17c0-4.2 2.5-7 5.5-7s5.5 2.8 5.5 7M12 17v-7" fill="none" stroke="#fff" stroke-width="1.9" stroke-linecap="round"/><circle cx="18.6" cy="5.4" r="2.3" fill="#FFC23A"/></svg>"##;

pub fn logo_rgba(size: u32) -> (Vec<u8>, u32) {
    let tree = resvg::usvg::Tree::from_str(LOGO_SVG, &resvg::usvg::Options::default()).expect("logo");
    let mut pm = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
    let s = size as f32 / tree.size().width();
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(s, s), &mut pm.as_mut());
    // tiny-skia menyimpan premultiplied; ubah ke RGBA biasa
    let mut v = pm.data().to_vec();
    for px in v.chunks_mut(4) {
        let a = px[3] as f32 / 255.0;
        if a > 0.0 { for c in &mut px[..3] { *c = ((*c as f32 / a).min(255.0)) as u8; } }
    }
    (v, size)
}
