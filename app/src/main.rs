//! Passerelle — aplikasi Windows native (Rust + egui), belajar bahasa Prancis offline.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod content;
mod lesson;
mod screens;
mod store;
mod ui;

use audio::{Audio, Recorder, Take};
use egui::{vec2, Align, Key, Layout, RichText};
use lesson::{Cx, DialogRun, Session};
use std::collections::HashMap;
use std::time::Instant;
use store::{State, Step};
use ui::{p, Kind};

#[derive(Clone, Copy, PartialEq)]
pub enum Screen { Home, Path, Speak, Dict, Progress, Settings }

#[derive(Clone)]
pub enum Confirm { ExitSession, SkipExam(u8), WipeAll, ClearRecs }

#[derive(Clone)]
pub enum Modal { Unit(&'static content::Unit), Note(&'static content::Unit), Why(String, String, Option<&'static content::Unit>), Confirm(Confirm), Quick(String) }

pub enum Act { Step(Step), Exam(u8, bool), Daily, Practice(String), Dialog(&'static content::Unit), Go(Screen), Dict(String) }

pub struct Onb { pub step: u8, pub name: String, pub rec: Option<Recorder>, pub msg: String }

#[derive(Clone, Copy, PartialEq)]
pub enum SpeakTab { Dialog, Shadow, Pairs, Recs }

pub struct App {
    pub st: State,
    pub au: Audio,
    pub screen: Screen,
    pub session: Option<Session>,
    pub dialog: Option<DialogRun>,
    pub onb: Option<Onb>,
    pub modal: Option<Modal>,
    pub pop: Option<(String, egui::Pos2)>,
    pop_seen: Option<(String, egui::Pos2)>,
    pub toasts: Vec<(String, Instant)>,
    pub acts: Vec<Act>,
    pub dict_q: String,
    pub dict_sel: Option<usize>,
    pub dict_tense: usize,
    pub speak_tab: SpeakTab,
    pub shadow_rec: Option<(String, Recorder)>,
    pub takes: HashMap<String, Take>,
    pub recs: Vec<audio::RecMeta>,
    pub pair_quiz: Option<(usize, bool)>,
    pub pair_msg: HashMap<usize, (bool, String)>,
    last_saved: String,
    last_save_t: Instant,
    applied: Option<(store::Settings, bool)>,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        trace("app: fonts");
        ui::install_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        trace("app: state");
        let st = State::load();
        trace("app: audio");
        let mut au = Audio::new();
        trace("app: audio ok");
        au.sound = st.settings.sound;
        au.slow_rate = st.settings.slow_rate;
        let onb = if st.profile.onboarded { None } else { Some(Onb { step: 0, name: st.profile.name.clone(), rec: None, msg: String::new() }) };
        App {
            st, au, screen: Screen::Home, session: None, dialog: None, onb, modal: None, pop: None, pop_seen: None, toasts: vec![], acts: vec![],
            dict_q: String::new(), dict_sel: None, dict_tense: 0, speak_tab: SpeakTab::Dialog, shadow_rec: None, takes: HashMap::new(),
            recs: vec![], pair_quiz: None, pair_msg: HashMap::new(), last_saved: String::new(), last_save_t: Instant::now(), applied: None,
        }
    }

    pub fn toast(&mut self, s: impl Into<String>) { self.toasts.push((s.into(), Instant::now())); }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        let sys_dark = matches!(ctx.system_theme(), Some(egui::Theme::Dark));
        let key = (self.st.settings.clone(), sys_dark);
        if self.applied.as_ref() == Some(&key) { return; }
        let dark = match self.st.settings.theme.as_str() { "dark" => true, "light" => false, _ => sys_dark };
        let pl = if dark { ui::DARK } else { ui::LIGHT };
        ui::set_pal(pl);
        ui::apply_visuals(ctx, &pl);
        ctx.set_zoom_factor(self.st.settings.font_scale);
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(if dark { egui::SystemTheme::Dark } else { egui::SystemTheme::Light }));
        if self.st.settings.reduce_motion { ctx.style_mut(|s| s.animation_time = 0.0); } else { ctx.style_mut(|s| s.animation_time = 0.17); }
        self.au.sound = self.st.settings.sound;
        self.au.slow_rate = self.st.settings.slow_rate;
        self.applied = Some(key);
    }

    fn autosave(&mut self, force: bool) {
        if !force && self.last_save_t.elapsed().as_secs_f32() < 2.0 { return; }
        self.last_save_t = Instant::now();
        let s = serde_json::to_string(&self.st).unwrap_or_default();
        if s != self.last_saved { self.st.save(); self.last_saved = s; }
    }

    pub fn start_step(&mut self, step: Step) {
        match step {
            Step::Lesson(u, i) => self.session = Some(Session::new(format!("{} · {}/5", u.title, i + 1), lesson::SKind::Lesson(u, i), lesson::build_lesson(u, i))),
            Step::Dialog(u) => self.dialog = Some(DialogRun::new(u)),
            Step::UnitReview(u) => self.session = Some(Session::new(format!("Ulasan unit · {}", u.title), lesson::SKind::UnitReview(u), lesson::build_unit_review(u))),
            Step::Exam(t) => self.session = Some(Session::new(format!("Ujian Tier {t}"), lesson::SKind::Exam(t, false), lesson::build_exam(t))),
            Step::None => self.toast("Unit berikutnya sedang ditulis. Sementara itu, kerjakan ulasan atau latihan bicara."),
        }
    }

    fn run_acts(&mut self) {
        for a in std::mem::take(&mut self.acts) {
            match a {
                Act::Step(s) => self.start_step(s),
                Act::Exam(t, skip) => self.session = Some(Session::new(format!("Ujian Tier {t}"), lesson::SKind::Exam(t, skip), lesson::build_exam(t))),
                Act::Daily => { let due = self.st.due_cards(); if !due.is_empty() { self.session = Some(Session::new("Ulasan hari ini".into(), lesson::SKind::Daily, lesson::build_daily(&due))); } }
                Act::Practice(tag) => { let qs = lesson::build_practice(&tag, &self.st); self.session = Some(Session::new(format!("Latih: {}", screens::tag_name(&tag)), lesson::SKind::Practice, qs)); }
                Act::Dialog(u) => self.dialog = Some(DialogRun::new(u)),
                Act::Go(s) => { self.screen = s; if s == Screen::Speak { self.recs = audio::recs_all(); } }
                Act::Dict(w) => { self.session = None; self.dialog = None; self.screen = Screen::Dict; self.dict_q = w.clone(); self.dict_sel = content::lookup(&w).and_then(|e| content::c().dict.iter().position(|x| std::ptr::eq(x, e))); }
            }
        }
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        let pl = p();
        egui::SidePanel::left("nav").exact_width(236.0).resizable(false)
            .frame(egui::Frame::new().fill(pl.paper).inner_margin(egui::Margin::symmetric(12, 16)).stroke(egui::Stroke::new(1.0, pl.line)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
                    egui::Image::from_bytes("bytes://logo.svg", ui::LOGO_SVG.as_bytes()).paint_at(ui, r);
                    ui.label(RichText::new("Passerelle").font(ui::display(20.0)));
                });
                ui.add_space(14.0);
                let items = [(Screen::Home, "Beranda", "home"), (Screen::Path, "Jalur", "path"), (Screen::Speak, "Bicara", "mic"), (Screen::Dict, "Kamus", "book"), (Screen::Progress, "Progres", "chart")];
                for (i, (s, label, ic)) in items.iter().enumerate() {
                    if nav_item(ui, label, ic, self.screen == *s, Some(&format!("Ctrl {}", i + 1))).clicked() { self.acts.push(Act::Go(*s)); }
                }
                // Bagian bawah: dua kartu kecil + Pengaturan, ditata atas-ke-bawah di ruang yang tersisa.
                let foot_h = 2.0 * 66.0 + 8.0 * 2.0 + 44.0 + 6.0 + 28.0;
                ui.add_space((ui.available_height() - foot_h).max(8.0));
                {
                    let (n, _) = self.st.streak();
                    ui::card().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui::icon(ui, "flame", 28.0, pl.sun_press);
                            ui.vertical(|ui| { ui.label(RichText::new(n.to_string()).font(ui::display(19.0))); ui::small(ui, "hari berturut-turut", pl.ink2); });
                        });
                    });
                    let today = self.st.today_day();
                    ui::card().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui::ring(ui, (today.sec / (self.st.profile.goal as f64 * 60.0)) as f32, 34.0, egui::Id::new("ring-side"));
                            ui.vertical(|ui| { ui.label(RichText::new(format!("{}", (today.sec / 60.0) as u32)).font(ui::display(19.0))); ui::small(ui, &format!("dari {} mnt hari ini", self.st.profile.goal), pl.ink2); });
                        });
                    });
                    ui.add_space(6.0);
                    if nav_item(ui, "Pengaturan", "sliders", self.screen == Screen::Settings, None).clicked() { self.acts.push(Act::Go(Screen::Settings)); }
                }
            });
    }

    fn word_pop(&mut self, ctx: &egui::Context) {
        let Some((word, pos)) = self.pop.clone() else { return };
        let pl = p();
        let e = content::lookup(&word);
        let mut close = false;
        let screen = ctx.screen_rect();
        let x = (pos.x - 160.0).clamp(12.0, screen.right() - 332.0);
        let resp = egui::Area::new(egui::Id::new("wordpop")).order(egui::Order::Foreground).fixed_pos(egui::pos2(x, pos.y + 6.0)).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).fill(pl.surface).corner_radius(egui::CornerRadius::same(ui::R)).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
                ui.set_width(300.0);
                match e {
                    Some(e) => {
                        ui.horizontal(|ui| {
                            ui::fr_label(ui, &e.w, 22.0);
                            if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { if !self.au.say(&e.w, false) { self.toasts.push(("Suara Prancis belum terpasang.".into(), Instant::now())); } }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| { ui::chip(ui, if e.g == "m" { "maskulin" } else if e.g == "f" { "feminin" } else { &e.pos }, pl.surface2, pl.ink2); });
                        });
                        ui::small(ui, &format!("/{}/ · dibaca “{}”", e.ipa, e.pron), pl.ink2);
                        if content::norm(&word) != e.key { ui::small(ui, &format!("“{}” adalah bentuk dari “{}”", word, e.w), pl.ink3); }
                        ui.label(e.tr.join("; "));
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            let saved = self.st.has_card(&e.w);
                            if ui::btn(ui, if saved { "Tersimpan" } else { "Simpan untuk ulasan" }, Some("bookmark"), Kind::Normal, false, !saved).clicked() {
                                self.st.add_card(&e.w, &e.tr[0], &e.pron, "");
                                self.toasts.push((format!("“{}” masuk ulasan berjarak.", e.w), Instant::now()));
                            }
                            if ui::btn(ui, "Buka di kamus", None, Kind::Ghost, false, true).clicked() { self.acts.push(Act::Dict(e.w.clone())); close = true; }
                        });
                    }
                    None => { ui::fr_label(ui, &word, 22.0); ui::muted(ui, "Kata ini belum ada di kamus lokal."); }
                }
            });
        });
        // Klik yang membuka kartu terjadi pada frame yang sama; jangan dihitung sebagai "klik di luar".
        let fresh = self.pop_seen.as_ref() != Some(&(word.clone(), pos));
        self.pop_seen = Some((word.clone(), pos));
        if close || (!fresh && resp.response.clicked_elsewhere()) || ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.pop.as_ref().map(|p| p.0 == word).unwrap_or(false) { self.pop = None; self.pop_seen = None; }
        }
    }

    fn toasts_ui(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|t| t.1.elapsed().as_secs_f32() < 3.4);
        if self.toasts.is_empty() { return; }
        let pl = p();
        let sr = ctx.screen_rect();
        egui::Area::new(egui::Id::new("toasts")).order(egui::Order::Tooltip).anchor(egui::Align2::CENTER_BOTTOM, vec2(0.0, -28.0)).interactable(false).show(ctx, |ui| {
            for (t, _) in &self.toasts {
                egui::Frame::new().fill(pl.ink).corner_radius(egui::CornerRadius::same(ui::R)).inner_margin(egui::Margin::symmetric(16, 10)).show(ui, |ui| {
                    ui.set_max_width((sr.width() - 80.0).min(560.0));
                    ui.label(RichText::new(t).font(ui::semi(14.0)).color(pl.paper));
                });
            }
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

fn nav_item(ui: &mut egui::Ui, label: &str, ic: &str, on: bool, kbd: Option<&str>) -> egui::Response {
    let pl = p();
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let bg = if on { pl.blue_soft } else if resp.hovered() { pl.surface2 } else { egui::Color32::TRANSPARENT };
    let fg = if on { pl.blue_ink } else if resp.hovered() { pl.ink } else { pl.ink2 };
    ui.painter().rect_filled(rect, egui::CornerRadius::same(ui::R), bg);
    ui::icon_at(ui, ic, egui::Rect::from_center_size(egui::pos2(rect.left() + 24.0, rect.center().y), egui::Vec2::splat(22.0)), fg);
    ui.painter().text(egui::pos2(rect.left() + 46.0, rect.center().y), egui::Align2::LEFT_CENTER, label, ui::semi(15.0), fg);
    if let Some(k) = kbd { ui.painter().text(egui::pos2(rect.right() - 10.0, rect.center().y), egui::Align2::RIGHT_CENTER, k, ui::semi(11.0), pl.ink3); }
    resp
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);
        let pl = p();

        // Pintasan global
        let (ctrl_k, nav_key) = ctx.input(|i| (
            i.modifiers.ctrl && i.key_pressed(Key::K),
            if i.modifiers.ctrl && i.key_pressed(Key::Comma) { Some(5) } else
            if i.modifiers.ctrl { [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5].iter().position(|k| i.key_pressed(*k)) } else { None }));
        let busy = self.session.is_some() || self.dialog.is_some() || self.onb.is_some();
        if ctrl_k && self.modal.is_none() && self.onb.is_none() { self.modal = Some(Modal::Quick(String::new())); }
        if let (Some(n), false, true) = (nav_key, busy, self.modal.is_none()) {
            self.acts.push(Act::Go([Screen::Home, Screen::Path, Screen::Speak, Screen::Dict, Screen::Progress, Screen::Settings][n]));
        }

        // Garis tiga warna tipis di atas
        egui::TopBottomPanel::top("tricolore").exact_height(3.0).frame(egui::Frame::new()).show(ctx, |ui| {
            let r = ui.max_rect(); let w = r.width() / 3.0;
            for (i, c) in [pl.blue, pl.line, pl.red].iter().enumerate() {
                ui.painter().rect_filled(egui::Rect::from_min_size(r.min + vec2(w * i as f32, 0.0), vec2(w + 1.0, 3.0)), 0.0, *c);
            }
        });

        if self.onb.is_some() {
            egui::CentralPanel::default().frame(egui::Frame::new().fill(pl.paper)).show(ctx, |ui| { self.onboarding(ui); });
        } else if self.session.is_some() {
            let mut out = lesson::Out::None;
            egui::CentralPanel::default().frame(egui::Frame::new().fill(pl.paper)).show(ctx, |ui| {
                let mut cx = Cx { st: &mut self.st, au: &mut self.au, pop: &mut self.pop, toast: &mut self.toasts };
                let o = self.session.as_mut().unwrap().ui(ui, &mut cx);
                if self.modal.is_none() { out = o; }
            });
            match out {
                lesson::Out::Close => self.session = None,
                lesson::Out::Continue => { self.session = None; let s = self.st.next_step(); self.start_step(s); }
                lesson::Out::ConfirmExit => self.modal = Some(Modal::Confirm(Confirm::ExitSession)),
                lesson::Out::Why(t, b, u) => self.modal = Some(Modal::Why(t, b, u)),
                lesson::Out::None => {}
            }
        } else if self.dialog.is_some() {
            let mut out = lesson::DOut::None;
            egui::CentralPanel::default().frame(egui::Frame::new().fill(pl.paper)).show(ctx, |ui| {
                let mut cx = Cx { st: &mut self.st, au: &mut self.au, pop: &mut self.pop, toast: &mut self.toasts };
                out = self.dialog.as_mut().unwrap().ui(ui, &mut cx);
            });
            match out {
                lesson::DOut::Close => { let d = self.dialog.take().unwrap(); self.st.add_time(0.0); let _ = d; }
                lesson::DOut::Done => { self.dialog = None; let s = self.st.next_step(); if matches!(s, Step::UnitReview(_)) { self.start_step(s); } }
                lesson::DOut::Again => { let u = self.dialog.take().unwrap().unit; self.dialog = Some(DialogRun::new(u)); }
                lesson::DOut::Skip => {
                    let u = self.dialog.take().unwrap().unit;
                    self.st.progress.dialog_skip_streak += 1;
                    let pr = self.st.up(&u.id); pr.dialog = true; pr.skipped = true;
                    self.toast("Dialog dilewati. Setelah dua kali berturut-turut, dialog berikutnya wajib diselesaikan.");
                }
                lesson::DOut::None => {}
            }
        } else {
            self.sidebar(ctx);
            egui::CentralPanel::default().frame(egui::Frame::new().fill(pl.paper)).show(ctx, |ui| {
                egui::ScrollArea::vertical().id_salt(self.screen as u8).auto_shrink([false, false]).show(ui, |ui| {
                    let w = 1040f32.min(ui.available_width() - 64.0);
                    ui.horizontal(|ui| {
                        ui.add_space(((ui.available_width() - w) / 2.0).max(32.0));
                        ui.vertical(|ui| {
                            ui.set_width(w);
                            ui.add_space(32.0);
                            match self.screen {
                                Screen::Home => self.home(ui),
                                Screen::Path => self.path(ui),
                                Screen::Speak => self.speak(ui),
                                Screen::Dict => self.dict(ui),
                                Screen::Progress => self.progress(ui),
                                Screen::Settings => self.settings(ui),
                            }
                            ui.add_space(64.0);
                        });
                    });
                });
            });
        }

        self.modals(ctx);
        self.word_pop(ctx);
        self.toasts_ui(ctx);
        self.run_acts();
        if self.shadow_rec.is_some() || (self.onb.as_ref().map(|o| o.rec.is_some()).unwrap_or(false)) { ctx.request_repaint(); }
        self.autosave(false);
    }

    fn on_exit(&mut self) {
        if let Some(s) = &self.session { let t = s.elapsed(); self.st.add_time(t); }
        self.autosave(true);
    }
}

/// Penanda diagnostik; hanya aktif bila PASSERELLE_TRACE diset.
pub fn trace(s: &str) {
    if std::env::var_os("PASSERELLE_TRACE").is_some() {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(store::data_dir().join("trace.log")) { let _ = writeln!(f, "{s}"); }
    }
}

mod gpu {
    include!("../../gpu.rs");
}

fn main() -> eframe::Result {
    trace("main: start");
    let (rgba, size) = ui::logo_rgba(64);
    trace("main: logo ok");
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Passerelle")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 640.0])
            .with_icon(egui::IconData { rgba, width: size, height: size }),
        centered: true,
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: gpu::wgpu_options(),
        ..Default::default()
    };
    trace("main: run_native");
    eframe::run_native("Passerelle", opts, Box::new(|cc| { trace("main: creator"); Ok(Box::new(App::new(cc))) }))
}
