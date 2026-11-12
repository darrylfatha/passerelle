//! Pemasang Passerelle (Rust + egui, native tanpa WebView).
//! - Menyalin Passerelle.exe ke %LOCALAPPDATA%\Programs\Passerelle (bisa diubah)
//! - Membuat shortcut Desktop dan Start Menu
//! - Mendaftarkan uninstaller di "Aplikasi & fitur" Windows (HKCU, tanpa hak admin)
//! Argumen: --silent (pasang tanpa jendela), --uninstall (lepas pasang).
#![windows_subsystem = "windows"]

use eframe::egui::{self, vec2, Color32, CornerRadius, FontFamily, FontId, Margin, RichText, Stroke};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

static APP_EXE: &[u8] = include_bytes!("../../target/release/Passerelle.exe");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Passerelle";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const BLUE: Color32 = Color32::from_rgb(0x2B, 0x54, 0xF5);
const BLUE_PRESS: Color32 = Color32::from_rgb(0x1A, 0x39, 0xC2);
const PAPER: Color32 = Color32::from_rgb(0xFF, 0xF9, 0xF0);
const INK: Color32 = Color32::from_rgb(0x1B, 0x1E, 0x2B);
const INK2: Color32 = Color32::from_rgb(0x58, 0x5F, 0x73);
const LINE: Color32 = Color32::from_rgb(0xEE, 0xE2, 0xCF);
const LINE2: Color32 = Color32::from_rgb(0xDC, 0xCB, 0xB0);
const SUN: Color32 = Color32::from_rgb(0xFF, 0xC2, 0x3A);
const RED: Color32 = Color32::from_rgb(0xF2, 0x3A, 0x48);
const RED_INK: Color32 = Color32::from_rgb(0xB4, 0x14, 0x1F);

fn default_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\")).join("Programs").join("Passerelle")
}
fn installed_dir() -> Option<PathBuf> {
    let k = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).open_subkey(UNINSTALL_KEY).ok()?;
    k.get_value::<String, _>("InstallLocation").ok().map(PathBuf::from)
}
fn ps_quote(s: &str) -> String { format!("'{}'", s.replace('\'', "''")) }

fn powershell(script: &str) -> Result<(), String> {
    let out = Command::new("powershell.exe").args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW).output().map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
}

struct Opts { dir: PathBuf, desktop: bool, start_menu: bool }

fn install(o: &Opts, log: &mut dyn FnMut(&str)) -> Result<PathBuf, String> {
    // Tutup Passerelle yang sedang berjalan agar berkasnya bisa diganti.
    let _ = Command::new("taskkill").args(["/IM", "Passerelle.exe", "/F"]).creation_flags(CREATE_NO_WINDOW).output();
    std::thread::sleep(std::time::Duration::from_millis(300));
    log("Menyiapkan folder…");
    std::fs::create_dir_all(&o.dir).map_err(|e| format!("Folder tidak bisa dibuat: {e}"))?;
    let exe = o.dir.join("Passerelle.exe");
    log("Menyalin aplikasi…");
    std::fs::write(&exe, APP_EXE).map_err(|e| format!("Passerelle.exe tidak bisa ditulis: {e}"))?;
    let un = o.dir.join("Uninstall Passerelle.exe");
    if let Ok(me) = std::env::current_exe() { if me != un { let _ = std::fs::copy(&me, &un); } }

    log("Membuat shortcut…");
    let mut script = String::from("$w = New-Object -ComObject WScript.Shell;");
    let mk = |folder: &str, sub: &str| format!(
        "$d = [Environment]::GetFolderPath('{folder}'){sub}; New-Item -ItemType Directory -Force $d | Out-Null; $s = $w.CreateShortcut((Join-Path $d 'Passerelle.lnk')); $s.TargetPath = {t}; $s.WorkingDirectory = {wd}; $s.IconLocation = {ic}; $s.Description = 'Belajar bahasa Prancis offline'; $s.Save();",
        t = ps_quote(&exe.to_string_lossy()), wd = ps_quote(&o.dir.to_string_lossy()), ic = ps_quote(&format!("{},0", exe.to_string_lossy())));
    if o.desktop { script.push_str(&mk("Desktop", "")); }
    if o.start_menu { script.push_str(&mk("Programs", "")); }
    if o.desktop || o.start_menu { powershell(&script).map_err(|e| format!("Shortcut gagal dibuat: {e}"))?; }

    log("Mendaftarkan di Windows…");
    let hk = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (k, _) = hk.create_subkey(UNINSTALL_KEY).map_err(|e| e.to_string())?;
    let size_kb = (APP_EXE.len() as u64 * 2 / 1024) as u32;
    let _ = k.set_value("DisplayName", &"Passerelle");
    let _ = k.set_value("DisplayVersion", &VERSION);
    let _ = k.set_value("Publisher", &"Passerelle");
    let _ = k.set_value("DisplayIcon", &format!("{},0", exe.to_string_lossy()));
    let _ = k.set_value("InstallLocation", &o.dir.to_string_lossy().to_string());
    let _ = k.set_value("UninstallString", &format!("\"{}\" --uninstall", un.to_string_lossy()));
    let _ = k.set_value("QuietUninstallString", &format!("\"{}\" --uninstall --silent", un.to_string_lossy()));
    let _ = k.set_value("EstimatedSize", &size_kb);
    let _ = k.set_value("NoModify", &1u32);
    let _ = k.set_value("NoRepair", &1u32);
    log("Selesai.");
    Ok(exe)
}

fn uninstall(remove_data: bool) -> Result<(), String> {
    let _ = Command::new("taskkill").args(["/IM", "Passerelle.exe", "/F"]).creation_flags(CREATE_NO_WINDOW).output();
    let dir = installed_dir().or_else(|| std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf))).ok_or("Folder pemasangan tidak ditemukan.")?;
    let _ = powershell("foreach ($f in 'Desktop','Programs') { $p = Join-Path ([Environment]::GetFolderPath($f)) 'Passerelle.lnk'; if (Test-Path $p) { Remove-Item $p -Force } }");
    let _ = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).delete_subkey_all(UNINSTALL_KEY);
    if remove_data {
        if let Some(a) = std::env::var_os("APPDATA") { let _ = std::fs::remove_dir_all(PathBuf::from(a).join("Passerelle")); }
    }
    let _ = std::fs::remove_file(dir.join("Passerelle.exe"));
    // Uninstaller sedang berjalan dari folder itu: hapus foldernya setelah proses ini keluar.
    let cmd = format!("ping 127.0.0.1 -n 3 > nul & rmdir /s /q \"{}\"", dir.to_string_lossy());
    let _ = Command::new("cmd.exe").args(["/C", &cmd]).creation_flags(CREATE_NO_WINDOW).spawn();
    Ok(())
}

/* ── Jendela ── */
#[derive(PartialEq)]
enum Stage { Ready, Done(PathBuf), Failed(String), UnReady, UnDone }

struct Setup { opts: Opts, path_text: String, stage: Stage, open_after: bool, remove_data: bool }

fn fonts(ctx: &egui::Context) {
    let mut defs = egui::FontDefinitions::default();
    let dir = PathBuf::from(std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into())).join("Fonts");
    for (k, f) in [("ui", "segoeui.ttf"), ("semi", "seguisb.ttf"), ("display", "seguibl.ttf"), ("serif", "georgia.ttf")] {
        if let Ok(b) = std::fs::read(dir.join(f)) { defs.font_data.insert(k.into(), std::sync::Arc::new(egui::FontData::from_owned(b))); }
    }
    let base = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (fam, k) in [(FontFamily::Proportional, "ui"), (FontFamily::Name("semi".into()), "semi"), (FontFamily::Name("display".into()), "display"), (FontFamily::Name("serif".into()), "serif")] {
        let mut v = vec![];
        if defs.font_data.contains_key(k) { v.push(k.to_string()); }
        if defs.font_data.contains_key("ui") { v.push("ui".into()); }
        v.extend(base.iter().cloned());
        defs.families.insert(fam, v);
    }
    ctx.set_fonts(defs);
    let mut v = egui::Visuals::light();
    v.panel_fill = PAPER; v.override_text_color = Some(INK);
    v.widgets.inactive.bg_fill = LINE2; v.widgets.inactive.weak_bg_fill = Color32::WHITE;
    v.selection.bg_fill = BLUE; v.selection.stroke = Stroke::new(2.0, Color32::WHITE);
    v.extreme_bg_color = Color32::WHITE;
    ctx.set_visuals(v);
    ctx.style_mut(|s| { s.spacing.item_spacing = vec2(8.0, 10.0); s.spacing.interact_size.y = 26.0; });
}
fn font(name: &str, size: f32) -> FontId { FontId::new(size, if name == "ui" { FontFamily::Proportional } else { FontFamily::Name(name.into()) }) }

fn big_btn(ui: &mut egui::Ui, label: &str, primary: bool) -> egui::Response {
    let g = ui.fonts(|f| f.layout_no_wrap(label.into(), font("semi", 16.0), if primary { Color32::WHITE } else { INK }));
    let (r, resp) = ui.allocate_exact_size(vec2(g.size().x + 44.0, 50.0), egui::Sense::click());
    let down = resp.is_pointer_button_down_on();
    let face = egui::Rect::from_min_size(r.min + vec2(0.0, if down { 3.0 } else { 0.0 }), vec2(r.width(), 47.0));
    let (fill, base) = if primary { (BLUE, BLUE_PRESS) } else { (Color32::WHITE, LINE2) };
    if !down { ui.painter().rect_filled(face.translate(vec2(0.0, 3.0)), CornerRadius::same(8), base); }
    ui.painter().rect(face, CornerRadius::same(8), if resp.hovered() && !primary { PAPER } else { fill }, Stroke::new(1.0, base), egui::StrokeKind::Inside);
    ui.painter().galley(face.center() - g.size() / 2.0, g, INK);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}
fn switch(ui: &mut egui::Ui, on: &mut bool, label: &str) {
    ui.horizontal(|ui| {
        let (rect, resp) = ui.allocate_exact_size(vec2(44.0, 26.0), egui::Sense::click());
        if resp.clicked() { *on = !*on; }
        ui.painter().rect_filled(rect, CornerRadius::same(13), if *on { BLUE } else { LINE2 });
        let x = if *on { rect.right() - 13.0 } else { rect.left() + 13.0 };
        ui.painter().circle_filled(egui::pos2(x, rect.center().y), 10.0, Color32::WHITE);
        if ui.add(egui::Label::new(RichText::new(label).font(font("ui", 15.0))).sense(egui::Sense::click())).clicked() { *on = !*on; }
    });
}

impl eframe::App for Setup {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tri").exact_height(4.0).frame(egui::Frame::new()).show(ctx, |ui| {
            let r = ui.max_rect(); let w = r.width() / 3.0;
            for (i, c) in [BLUE, LINE, RED].iter().enumerate() { ui.painter().rect_filled(egui::Rect::from_min_size(r.min + vec2(w * i as f32, 0.0), vec2(w + 1.0, 4.0)), 0.0, *c); }
        });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(PAPER).inner_margin(Margin::symmetric(36, 28))).show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(56.0, 56.0), egui::Sense::hover());
                egui::Image::from_bytes("bytes://logo.svg", LOGO.as_bytes()).paint_at(ui, r);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    let title = match self.stage { Stage::UnReady | Stage::UnDone => "Lepas pasang Passerelle", _ => "Pasang Passerelle" };
                    ui.label(RichText::new(title).font(font("display", 26.0)));
                    ui.label(RichText::new(format!("Versi {VERSION} · belajar bahasa Prancis, offline")).font(font("ui", 14.0)).color(INK2));
                });
            });
            ui.add_space(18.0);
            match &self.stage {
                Stage::Ready => {
                    ui.label(RichText::new("Folder pemasangan").font(font("semi", 14.5)));
                    let r = ui.add(egui::TextEdit::singleline(&mut self.path_text).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8)).font(font("ui", 14.0)));
                    if r.changed() { self.opts.dir = PathBuf::from(self.path_text.trim()); }
                    ui.label(RichText::new("Tidak butuh hak administrator. Progres belajar disimpan terpisah, jadi aman saat memperbarui.").font(font("ui", 12.5)).color(INK2));
                    ui.add_space(8.0);
                    switch(ui, &mut self.opts.desktop, "Buat shortcut di Desktop");
                    switch(ui, &mut self.opts.start_menu, "Tambahkan ke Start Menu");
                    switch(ui, &mut self.open_after, "Buka Passerelle setelah selesai");
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if big_btn(ui, "Pasang", true).clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            match install(&self.opts, &mut |_| {}) {
                                Ok(exe) => { if self.open_after { let _ = Command::new(&exe).current_dir(&self.opts.dir).spawn(); } self.stage = Stage::Done(exe); }
                                Err(e) => self.stage = Stage::Failed(e),
                            }
                        }
                        if big_btn(ui, "Batal", false).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                    });
                }
                Stage::Done(exe) => {
                    let exe = exe.clone();
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(40.0, 40.0), egui::Sense::hover());
                        ui.painter().circle_filled(r.center(), 20.0, SUN);
                        ui.painter().line_segment([r.center() + vec2(-8.0, 0.0), r.center() + vec2(-2.0, 6.0)], Stroke::new(3.0, INK));
                        ui.painter().line_segment([r.center() + vec2(-2.0, 6.0), r.center() + vec2(9.0, -7.0)], Stroke::new(3.0, INK));
                        ui.label(RichText::new("Passerelle sudah terpasang.").font(font("display", 20.0)));
                    });
                    ui.label(RichText::new(format!("Lokasi: {}", exe.to_string_lossy())).font(font("ui", 13.0)).color(INK2));
                    if self.opts.desktop { ui.label(RichText::new("Shortcut “Passerelle” ada di Desktop.").font(font("ui", 14.5))); }
                    ui.label(RichText::new("Untuk melepas pasang: Pengaturan Windows → Aplikasi → Passerelle.").font(font("ui", 13.0)).color(INK2));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if big_btn(ui, "Buka Passerelle", true).clicked() { let _ = Command::new(&exe).spawn(); ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                        if big_btn(ui, "Tutup", false).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                    });
                }
                Stage::Failed(e) => {
                    ui.label(RichText::new("Pemasangan gagal").font(font("display", 20.0)).color(RED_INK));
                    ui.label(RichText::new(e).font(font("ui", 14.0)));
                    ui.add_space(12.0);
                    if big_btn(ui, "Coba lagi", true).clicked() { self.stage = Stage::Ready; }
                }
                Stage::UnReady => {
                    ui.label(RichText::new("Passerelle, shortcut, dan pendaftarannya di Windows akan dihapus.").font(font("ui", 15.0)));
                    switch(ui, &mut self.remove_data, "Hapus juga progres belajar dan rekaman suara");
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if big_btn(ui, "Lepas pasang", true).clicked() {
                            match uninstall(self.remove_data) { Ok(()) => self.stage = Stage::UnDone, Err(e) => self.stage = Stage::Failed(e) }
                        }
                        if big_btn(ui, "Batal", false).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                    });
                }
                Stage::UnDone => {
                    ui.label(RichText::new("Passerelle sudah dilepas. Au revoir !").font(font("display", 20.0)));
                    ui.add_space(12.0);
                    if big_btn(ui, "Tutup", true).clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                }
            }
        });
    }
}

const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="256" height="256"><rect width="24" height="24" rx="5.5" fill="#2B54F5"/><path d="M4 17h16M6.5 17c0-4.2 2.5-7 5.5-7s5.5 2.8 5.5 7M12 17v-7" fill="none" stroke="#fff" stroke-width="1.9" stroke-linecap="round"/><circle cx="18.6" cy="5.4" r="2.3" fill="#FFC23A"/></svg>"##;

fn logo_icon() -> egui::IconData {
    let tree = resvg::usvg::Tree::from_str(LOGO, &resvg::usvg::Options::default()).unwrap();
    let mut pm = resvg::tiny_skia::Pixmap::new(64, 64).unwrap();
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(64.0 / tree.size().width(), 64.0 / tree.size().width()), &mut pm.as_mut());
    let mut rgba = pm.data().to_vec();
    for px in rgba.chunks_mut(4) { let a = px[3] as f32 / 255.0; if a > 0.0 { for c in &mut px[..3] { *c = ((*c as f32 / a).min(255.0)) as u8; } } }
    egui::IconData { rgba, width: 64, height: 64 }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let silent = args.iter().any(|a| a == "--silent");
    let un = args.iter().any(|a| a == "--uninstall");
    if silent {
        if un { let _ = uninstall(false); }
        else { let _ = install(&Opts { dir: default_dir(), desktop: true, start_menu: true }, &mut |_| {}); }
        return Ok(());
    }
    let dir = installed_dir().unwrap_or_else(default_dir);
    let setup = Setup { path_text: dir.to_string_lossy().to_string(), opts: Opts { dir, desktop: true, start_menu: true }, stage: if un { Stage::UnReady } else { Stage::Ready }, open_after: true, remove_data: false };
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title(if un { "Lepas pasang Passerelle" } else { "Pasang Passerelle" })
            .with_inner_size([620.0, 470.0]).with_resizable(false).with_maximize_button(false).with_icon(logo_icon()),
        centered: true,
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: gpu::wgpu_options(),
        ..Default::default()
    };
    eframe::run_native("Passerelle Setup", opts, Box::new(|cc| {
        egui_extras_svg(&cc.egui_ctx);
        fonts(&cc.egui_ctx);
        Ok(Box::new(setup))
    }))
}

mod gpu {
    include!("../../gpu.rs");
}

fn egui_extras_svg(ctx: &egui::Context) { egui_extras::install_image_loaders(ctx); }
