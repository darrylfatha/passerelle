//! Layar utama Passerelle.
use crate::audio::{self, Recorder};
use crate::content::{self, Unit};
use crate::lesson;
use crate::store::{self, Step};
use crate::ui::{self, p, Kind};
use crate::{Act, App, Confirm, Modal, Screen, SpeakTab};
use egui::{vec2, Align, Color32, Layout, RichText, Ui};

pub fn tag_name(t: &str) -> &str {
    match t { "gender" => "Gender (maskulin/feminin)", "konjugasi" => "Konjugasi kata kerja", "preposisi" => "Preposisi", "urutan kata" => "Urutan kata",
              "ejaan" => "Ejaan", "aksen" => "Aksen", "kosakata" => "Kosakata", "pelafalan" => "Pelafalan", x => x }
}
fn step_label(s: &Step) -> String {
    match s { Step::Lesson(_, i) => format!("Pelajaran {} dari 5 · sekitar 5–8 menit", i + 1), Step::Dialog(_) => "Dialog latihan".into(), Step::UnitReview(_) => "Ulasan unit".into(),
              Step::Exam(_) => "Membaca, mendengar, menulis, dan berbicara. Lulus di 80%.".into(), Step::None => String::new() }
}
fn head(ui: &mut Ui, title: &str, sub: &str) { ui::h1(ui, title); if !sub.is_empty() { ui::muted(ui, sub); } ui.add_space(18.0); }
fn mastery_name(m: u8) -> &'static str { ["Belum mulai", "Dikenal", "Terlatih", "Dikuasai"][m as usize] }

impl App {
    /* ══════════ BERANDA ══════════ */
    pub fn home(&mut self, ui: &mut Ui) {
        let pl = p();
        let step = self.st.next_step();
        let hour = chrono::Timelike::hour(&chrono::Local::now());
        let greet = if hour < 11 { "Bonjour" } else if hour < 18 { "Salut" } else { "Bonsoir" };
        // Tipografi Prancis: spasi sempit sebelum tanda seru.
        let name = if self.st.profile.name.is_empty() { String::new() } else { format!(", {}", self.st.profile.name) };
        ui.label(RichText::new(format!("{greet}{name}\u{202f}!")).font(ui::serif(36.0)).strong());
        if step != Step::None { ui::muted(ui, "Satu langkah berikutnya sudah siap."); }
        ui.add_space(18.0);
        let full = ui.available_width();
        let lw = full * 0.61;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(lw);
                self.hero(ui, step, lw);
                ui.add_space(12.0);
                ui::cols(ui, 2, |c| {
                    let due = self.st.due_cards();
                    ui::card().show(&mut c[0], |ui| {
                        ui.set_min_height(150.0);
                        ui.horizontal(|ui| { ui::h3(ui, "Ulasan hari ini"); ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if due.is_empty() { ui::chip(ui, "Beres", pl.blue_soft, pl.blue_ink); } else { ui::chip(ui, &format!("{} kartu", due.len()), pl.sun_soft, pl.sun_ink); } }); });
                        ui::small(ui, if !due.is_empty() { "Kata yang hampir terlupa. Lima menit cukup." } else if !self.st.cards.is_empty() { "Tidak ada kartu jatuh tempo. Kartu berikutnya muncul besok." } else { "Kartu muncul setelah pelajaran pertama." }, pl.ink2);
                        ui.add_space(8.0);
                        if ui::btn(ui, "Mulai ulasan", Some("repeat"), Kind::Normal, false, !due.is_empty()).clicked() { self.acts.push(Act::Daily); }
                    });
                    let cs = content::c();
                    let d = &cs.daily[(chrono::Local::now().timestamp() / 86400) as usize % cs.daily.len()];
                    egui::Frame::new().fill(pl.surface).stroke(egui::Stroke::new(1.0, pl.line)).corner_radius(egui::CornerRadius::same(ui::R)).inner_margin(egui::Margin::same(20)).show(&mut c[1], |ui| {
                        ui.set_min_height(150.0);
                        let r = ui.max_rect();
                        ui.painter().rect_filled(egui::Rect::from_min_size(r.min - vec2(20.0, 20.0), vec2(4.0, r.height() + 40.0)), 0.0, pl.sun);
                        ui::eyebrow(ui, "Kalimat hari ini", pl.ink2);
                        ui.horizontal(|ui| {
                            if let Some(w) = ui::fr_words(ui, &d.fr, 20.0, pl.ink) { self.pop = Some(w); }
                        });
                        ui.horizontal(|ui| { ui.label(&d.tr); if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { self.say(&d.fr.clone(), false); } });
                        ui::small(ui, &d.note, pl.ink3);
                    });
                });
            });
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.set_width(full - lw - 14.0);
                let today = self.st.today_day();
                let goal = self.st.profile.goal as f64 * 60.0;
                ui::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui::ring(ui, (today.sec / goal) as f32, 84.0, egui::Id::new("ring-home"));
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui::eyebrow(ui, "Target harian", pl.ink2);
                            ui.horizontal(|ui| { ui.label(RichText::new(format!("{}", (today.sec / 60.0) as u32)).font(ui::display(30.0))); ui.label(RichText::new(format!("/ {} mnt", self.st.profile.goal)).color(pl.ink2)); });
                            ui::small(ui, &if today.sec >= goal { "Target tercapai. Mantap.".to_string() } else { format!("{} menit lagi", (((goal - today.sec) / 60.0).ceil() as u32).max(1)) }, pl.ink2);
                        });
                    });
                });
                ui.add_space(12.0);
                let (n, freezes) = self.st.streak();
                ui::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| { ui::icon(ui, "flame", 24.0, pl.sun_press); ui::h3(ui, &format!("{n} hari berturut-turut")); });
                    ui::small(ui, "Dua hari jeda per minggu, jadi satu hari sibuk tidak memutus rangkaian.", pl.ink2);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let mon = store::week_start(store::today());
                        for (i, dn) in ["Sn", "Sl", "Rb", "Km", "Jm", "Sb", "Mg"].iter().enumerate() {
                            let d = mon + chrono::Duration::days(i as i64);
                            let k = store::dkey(d);
                            let on = self.st.days.get(&k).map(|x| x.sec >= 60.0).unwrap_or(false);
                            let fz = freezes.contains(&k);
                            ui.vertical(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
                                let c = r.center();
                                if on { ui.painter().circle_filled(c, 13.0, pl.sun); ui::icon_at(ui, "check", r.shrink(7.0), ui::LIGHT.ink); }
                                else if fz { ui.painter().circle(c, 12.0, pl.blue_soft, egui::Stroke::new(2.0, pl.blue)); ui::icon_at(ui, "slow", r.shrink(7.0), pl.blue_ink); }
                                else { ui.painter().circle_stroke(c, 12.0, egui::Stroke::new(1.5, pl.line2)); }
                                ui.label(RichText::new(*dn).font(ui::bold(11.0)).color(if d == store::today() { pl.ink } else { pl.ink3 }));
                            });
                        }
                    });
                });
                ui.add_space(12.0);
                let m = self.st.metrics();
                ui::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui::eyebrow(ui, "Kata dikuasai", pl.ink2);
                    ui.horizontal(|ui| { ui.label(RichText::new(m.mastered.to_string()).font(ui::display(40.0))); ui.label(RichText::new(format!("dari {} kata di ulasan", m.cards)).color(pl.ink2)); });
                    ui::small(ui, "Dihitung dari kata yang sudah 4× benar berturut-turut di ulasan berjarak.", pl.ink3);
                });
            });
        });
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) && self.modal.is_none() && ui.ctx().memory(|m| m.focused().is_none()) { self.acts.push(Act::Step(step)); }
    }

    fn hero(&mut self, ui: &mut Ui, step: Step, w: f32) {
        let pl = p();
        let t = match step { Step::Lesson(u, _) | Step::Dialog(u) | Step::UnitReview(u) => u.tier, Step::Exam(t) => t, Step::None => 1 } as usize;
        let resp = egui::Frame::new().fill(pl.tier_soft[t]).corner_radius(egui::CornerRadius::same(ui::R)).inner_margin(egui::Margin::same(32)).show(ui, |ui| {
            ui.set_width(w - 64.0);
            let r = ui.max_rect();
            // lingkaran dekoratif tier (hanya garis tepi, warna tier)
            let painter = ui.painter().with_clip_rect(r.expand(32.0));
            painter.circle_stroke(egui::pos2(r.right() + 10.0, r.top() + 30.0), 96.0, egui::Stroke::new(28.0, pl.tier[t].gamma_multiply(0.18)));
            painter.circle_stroke(egui::pos2(r.right() - 150.0, r.bottom() - 10.0), 38.0, egui::Stroke::new(13.0, pl.tier[t].gamma_multiply(0.14)));
            match step {
                Step::None => {
                    ui::eyebrow(ui, "Semua konten yang ada sudah kamu selesaikan", pl.tier_ink[t]);
                    ui::h1(ui, "Unit berikutnya sedang ditulis");
                    ui::muted(ui, "Jaga ingatanmu dengan ulasan hari ini dan latihan bicara.");
                    ui.add_space(10.0);
                    if ui::btn(ui, "Latihan bicara", Some("mic"), Kind::Primary, true, true).clicked() { self.acts.push(Act::Go(Screen::Speak)); }
                }
                _ => {
                    let unit = match step { Step::Lesson(u, _) | Step::Dialog(u) | Step::UnitReview(u) => Some(u), _ => None };
                    ui::eyebrow(ui, &format!("{}{}", ui::tier_name(t as u8), unit.map(|u| format!(" · {}", u.theme)).unwrap_or_default()), pl.tier_ink[t]);
                    ui.add_space(2.0);
                    match unit { Some(u) => { ui.label(RichText::new(&u.title).font(ui::serif(40.0)).strong()); } None => ui::h1(ui, &format!("Ujian Tier {t}")) }
                    ui::muted(ui, &step_label(&step));
                    if let Some(u) = unit {
                        let pr = self.st.upr(&u.id);
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            let mut seg = |ui: &mut Ui, on: bool, now: bool| {
                                let (r, _) = ui.allocate_exact_size(vec2(30.0, 9.0), egui::Sense::hover());
                                if on { ui.painter().rect_filled(r, egui::CornerRadius::same(4), pl.tier[t]); }
                                else { ui.painter().rect(r, egui::CornerRadius::same(4), pl.surface, egui::Stroke::new(if now { 2.0 } else { 1.0 }, if now { pl.tier[t] } else { pl.line2 }), egui::StrokeKind::Inside); }
                            };
                            for i in 0..5 { seg(ui, pr.lessons[i], step == Step::Lesson(u, i)); }
                            seg(ui, pr.dialog, step == Step::Dialog(u));
                            seg(ui, pr.review, step == Step::UnitReview(u));
                        });
                    }
                    ui.add_space(14.0);
                    if ui::btn(ui, "Lanjutkan", Some("right"), Kind::Primary, true, true).clicked() { self.acts.push(Act::Step(step)); }
                }
            }
        });
        let _ = resp;
    }

    pub fn say(&mut self, t: &str, slow: bool) {
        if !self.au.say(t, slow) { self.toast("Suara Prancis belum terpasang di Windows. Cara memasangnya ada di Pengaturan."); }
    }

    /* ══════════ JALUR ══════════ */
    pub fn path(&mut self, ui: &mut Ui) {
        let pl = p();
        head(ui, "Jalur belajar", "Tujuh tier, dari langkah pertama sampai fasih. Selesaikan unit untuk membuka unit berikutnya.");
        let next = self.st.next_step();
        let cs = content::c();
        for t in &cs.tiers {
            let ti = t.n as usize;
            let unlocked = self.st.tier_unlocked(t.n);
            let passed = self.st.tier_passed(t.n);
            let units = content::units_of_tier(t.n);
            egui::Frame::new().fill(if unlocked { pl.tier_soft[ti] } else { pl.surface2 }).corner_radius(egui::CornerRadius::same(ui::R)).inner_margin(egui::Margin::symmetric(20, 16)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(46.0, 46.0), egui::Sense::hover());
                    ui.painter().circle_filled(r.center(), 23.0, if unlocked { pl.tier[ti] } else { pl.line2 });
                    if unlocked { ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, t.n.to_string(), ui::display(20.0), Color32::WHITE); }
                    else { ui::icon_at(ui, "lock", r.shrink(13.0), Color32::WHITE); }
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui::eyebrow(ui, &format!("Tier {} · {}", t.n, t.label), if unlocked { pl.tier_ink[ti] } else { pl.ink3 });
                        ui.label(RichText::new(&t.name).font(ui::serif(24.0)).strong());
                        ui::small(ui, &t.goal, pl.ink2);
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(format!("{} unit", t.units)).font(ui::bold(12.5)).color(pl.ink2));
                        if passed { ui::chip(ui, "✓ Lulus", pl.blue_soft, pl.blue_ink); }
                        else if unlocked && !units.is_empty() && ui::btn(ui, "Ujian lompat", Some("trophy"), Kind::Normal, false, true).clicked() { self.modal = Some(Modal::Confirm(Confirm::SkipExam(t.n))); }
                    });
                });
            });
            ui.add_space(14.0);
            if unlocked {
                let mut row = 0;
                let indent = |r: usize| [0.0, 0.18, 0.32, 0.18][r % 4];
                let full = ui.available_width();
                for u in &units {
                    let open = self.st.unit_unlocked(u);
                    let done = self.st.unit_done(&u.id);
                    let steps = self.st.unit_steps(&u.id);
                    let cur = matches!(next, Step::Lesson(x, _) | Step::Dialog(x) | Step::UnitReview(x) if x.id == u.id);
                    ui.horizontal(|ui| {
                        ui.add_space(full * indent(row));
                        let r = node(ui, ti, if done { "star" } else if !open { "lock" } else if cur { "play" } else { "book" }, open, cur, steps as f32 / 7.0);
                        if r.clicked() && open { self.modal = Some(Modal::Unit(content::unit(&u.id).unwrap())); }
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.add_space(14.0);
                            ui.label(RichText::new(&u.title).font(ui::serif(20.0)).strong());
                            ui::small(ui, &format!("{}{}", u.theme, if open { format!(" · {steps}/7 langkah") } else { String::new() }), pl.ink2);
                            if done { ui::chip(ui, mastery_name(self.st.unit_mastery(u).max(1)), pl.tier_soft[ti], pl.tier_ink[ti]); }
                        });
                    });
                    ui.add_space(10.0);
                    row += 1;
                }
                if t.n == 1 {
                    for title in &cs.upcoming {
                        ui.horizontal(|ui| {
                            ui.add_space(full * indent(row));
                            node(ui, ti, "lock", false, false, 0.0);
                            ui.add_space(10.0);
                            ui.vertical(|ui| { ui.add_space(18.0); ui.label(RichText::new(title).font(ui::serif(19.0)).color(pl.ink3)); ui::small(ui, "Konten sedang ditulis", pl.ink3); });
                        });
                        ui.add_space(10.0);
                        row += 1;
                    }
                }
                if !units.is_empty() {
                    let ready = self.st.tier_exam_ready(t.n);
                    ui.horizontal(|ui| {
                        ui.add_space(full * indent(row));
                        if ui::btn(ui, if passed { "Ulangi ujian tier" } else { "Ujian tier" }, Some("trophy"), if ready && !passed { Kind::Sun } else { Kind::Normal }, true, ready || passed).clicked() { self.acts.push(Act::Exam(t.n, false)); }
                        ui::small(ui, if ready || passed { "Lulus di skor 80%." } else { "Terbuka setelah semua unit selesai." }, pl.ink2);
                    });
                }
                ui.add_space(36.0);
            }
        }
    }

    /* ══════════ BICARA ══════════ */
    pub fn speak(&mut self, ui: &mut Ui) {
        let pl = p();
        head(ui, "Bicara", "Semua rekaman tetap di komputer ini. Dengar contoh, ucapkan, bandingkan.");
        ui.horizontal(|ui| {
            for (t, l) in [(SpeakTab::Dialog, "Dialog bercabang"), (SpeakTab::Shadow, "Tiru-ucap"), (SpeakTab::Pairs, "Pasangan bunyi"), (SpeakTab::Recs, "Rekamanku")] {
                if ui::btn(ui, l, None, if self.speak_tab == t { Kind::Primary } else { Kind::Ghost }, false, true).clicked() { self.speak_tab = t; if t == SpeakTab::Recs { self.recs = audio::recs_all(); } }
            }
        });
        ui.add_space(14.0);
        let units: Vec<&'static Unit> = content::c().units.iter().filter(|u| self.st.unit_unlocked(u) || self.st.unit_done(&u.id)).collect();
        match self.speak_tab {
            SpeakTab::Dialog => {
                let first = units.iter().find(|u| { let pr = self.st.upr(&u.id); !pr.dialog && pr.lessons.iter().all(|x| *x) }).map(|u| u.id.clone());
                let cw = (ui.available_width() - 14.0) / 2.0;
                egui::Grid::new("dlg-grid").num_columns(2).spacing(vec2(14.0, 14.0)).show(ui, |ui| {
                    for (i, u) in units.iter().enumerate() {
                        let pr = self.st.upr(&u.id);
                        let avail = pr.dialog || pr.lessons.iter().all(|x| *x);
                        ui::card().show(ui, |ui| { ui.vertical(|ui| {
                            ui.set_width(cw - 40.0);
                            ui.horizontal(|ui| { ui::chip(ui, &ui::tier_name(u.tier), pl.tier_soft[u.tier as usize], pl.tier_ink[u.tier as usize]); if pr.dialog { ui::chip(ui, "✓ Selesai", pl.blue_soft, pl.blue_ink); } });
                            ui.add_space(4.0);
                            ui::h3(ui, &u.dialog.title);
                            ui::small(ui, &format!("Bersama {}, {}", u.dialog.partner, u.dialog.role), pl.ink2);
                            ui.label(RichText::new(format!("« {} »", u.dialog.nodes[&u.dialog.start].say)).font(ui::serif(17.0)).color(pl.ink2));
                            ui.add_space(6.0);
                            let primary = first.as_deref() == Some(u.id.as_str());
                            if ui::btn(ui, if !avail { "Selesaikan 5 pelajaran dulu" } else if primary { "Mulai dialog" } else { "Main lagi" }, Some(if avail { "chat" } else { "lock" }), if primary { Kind::Primary } else { Kind::Normal }, false, avail).clicked() {
                                self.acts.push(Act::Dialog(u));
                            }
                        }); });
                        if i % 2 == 1 { ui.end_row(); }
                    }
                });
            }
            SpeakTab::Shadow => {
                ui::muted(ui, "Dengar satu kalimat, rekam dirimu, lalu dengar keduanya berdampingan.");
                for u in &units {
                    ui.add_space(14.0);
                    ui.label(RichText::new(&u.title).font(ui::serif(20.0)).strong());
                    ui::card().inner_margin(egui::Margin::symmetric(20, 6)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (i, s) in u.sentences.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.set_min_height(56.0);
                                if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { self.say(&s.fr, false); }
                                ui.vertical(|ui| { ui.add_space(6.0); if let Some(w) = ui::fr_words(ui, &s.fr, 19.0, pl.ink) { self.pop = Some(w); } ui::small(ui, &s.tr, pl.ink2); });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| { self.rec_widget(ui, &s.fr, "tiru"); });
                            });
                            if i + 1 < u.sentences.len() { ui.separator(); }
                        }
                    });
                }
            }
            SpeakTab::Pairs => {
                ui::muted(ui, "Bunyi yang sulit bagi penutur Indonesia. Klik kata untuk mendengar, rekam dirimu mengucapkan keduanya, lalu bandingkan.");
                ui.add_space(10.0);
                let pairs = &content::c().pairs;
                let cw = (ui.available_width() - 28.0) / 3.0;
                egui::Grid::new("pairs").num_columns(3).spacing(vec2(14.0, 14.0)).show(ui, |ui| {
                    for (i, pr) in pairs.iter().enumerate() {
                        ui::card().show(ui, |ui| { ui.vertical(|ui| {
                            ui.set_width(cw - 40.0);
                            ui::cols(ui, 2, |c| {
                                for (k, (w, t)) in [(&pr.a, &pr.ta), (&pr.b, &pr.tb)].iter().enumerate() {
                                    let cw2 = c[k].available_width();
                                    let r = ui::opt(&mut c[k], None, &format!("{w}\n{t}"), true, ui::OptState::Normal, cw2, true);
                                    if r.clicked() {
                                        if let Some((qi, target_b)) = self.pair_quiz.filter(|q| q.0 == i) {
                                            let ok = (k == 1) == target_b;
                                            let heard = if target_b { pr.b.clone() } else { pr.a.clone() };
                                            self.pair_msg.insert(i, (ok, if ok { "✓ Benar".into() } else { format!("✗ Belum tepat, tadi “{heard}”") }));
                                            if ok { self.au.ok() } else { self.au.bad() }
                                            self.st.record("listen", "pelafalan", ok);
                                            let _ = qi; self.pair_quiz = None;
                                        } else { let w = w.to_string(); self.say(&w, false); }
                                    }
                                }
                            });
                            ui::small(ui, &pr.tip, pl.ink2);
                            ui.horizontal(|ui| {
                                self.rec_widget(ui, &format!("{} … {}", pr.a, pr.b), "tiru");
                                if self.au.has_voice() && ui::btn(ui, "Tebak bunyinya", Some("ear"), Kind::Normal, false, true).clicked() {
                                    let b = rand::random::<bool>();
                                    self.pair_quiz = Some((i, b));
                                    let w = if b { pr.b.clone() } else { pr.a.clone() };
                                    self.say(&w, false);
                                    self.pair_msg.insert(i, (true, "Mana yang terdengar? Klik kartunya.".into()));
                                }
                            });
                            if let Some((ok, m)) = self.pair_msg.get(&i) { ui.label(RichText::new(m).font(ui::bold(13.5)).color(if *ok { pl.blue_ink } else { pl.red_ink })); }
                        }); });
                        if i % 3 == 2 { ui.end_row(); }
                    }
                });
            }
            SpeakTab::Recs => {
                let total: f32 = self.recs.iter().map(|r| r.sec).sum();
                ui::card().inner_margin(egui::Margin::symmetric(20, 10)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if self.recs.is_empty() { ui.add_space(10.0); ui::muted(ui, "Belum ada rekaman. Coba tiru-ucap atau dialog."); ui.add_space(10.0); return; }
                    ui.horizontal(|ui| {
                        ui::h3(ui, &format!("{} rekaman · total {:.0} detik", self.recs.len(), total));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| { if ui::btn(ui, "Hapus semua", Some("trash"), Kind::Danger, false, true).clicked() { self.modal = Some(Modal::Confirm(Confirm::ClearRecs)); } });
                    });
                    ui.separator();
                    let mut del = None;
                    for r in self.recs.iter().rev() {
                        ui.horizontal(|ui| {
                            if ui::icon_btn(ui, "play", "Putar", Kind::Ghost).clicked() { if let Some(t) = audio::rec_load(r) { self.au.play_take(&t); } }
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&r.text).font(ui::serif(17.0)));
                                let when = chrono::DateTime::from_timestamp_millis(r.id).map(|d| d.with_timezone(&chrono::Local).format("%d/%m/%Y %H:%M").to_string()).unwrap_or_default();
                                ui::small(ui, &format!("{when} · {:.1} dtk · {}", r.sec, if r.kind == "dialog" { "dialog" } else { "tiru-ucap" }), pl.ink3);
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| { if ui::icon_btn(ui, "trash", "Hapus rekaman", Kind::Ghost).clicked() { del = Some(r.id); } });
                        });
                    }
                    if let Some(id) = del { audio::rec_delete(id); self.recs = audio::recs_all(); }
                });
            }
        }
    }

    fn rec_widget(&mut self, ui: &mut Ui, text: &str, kind: &str) {
        let key = text.to_string();
        let recording = self.shadow_rec.as_ref().map(|r| r.0 == key).unwrap_or(false);
        let has = self.takes.contains_key(&key);
        // tata letak kanan-ke-kiri: tombol putar dulu, lalu rekam
        if has && ui::icon_btn(ui, "play", "Putar rekamanku", Kind::Ghost).clicked() { let t = self.takes[&key].clone(); self.au.play_take(&t); }
        if ui::icon_btn(ui, if recording { "stop" } else { "mic" }, if recording { "Berhenti" } else { "Rekam" }, if recording { Kind::Danger } else { Kind::Ghost }).clicked() {
            if recording {
                let (_, r) = self.shadow_rec.take().unwrap();
                let t = r.stop();
                let mut cx = lesson::Cx { st: &mut self.st, au: &mut self.au, pop: &mut self.pop, toast: &mut self.toasts };
                lesson::save_take(&mut cx, text, &t, kind);
                self.au.play_take(&t);
                self.takes.insert(key, t);
            } else {
                self.shadow_rec = None;
                match Recorder::start() { Ok(r) => self.shadow_rec = Some((key, r)), Err(_) => self.toast("Mikrofon tidak bisa dipakai. Periksa izin mikrofon di Pengaturan Windows.") }
            }
        }
        if recording { ui.label(RichText::new("●").color(p().red)); }
    }

    /* ══════════ KAMUS ══════════ */
    pub fn dict(&mut self, ui: &mut Ui) {
        let pl = p();
        head(ui, "Kamus", "Prancis ↔ Indonesia, tanpa internet. Buka dari mana pun dengan Ctrl+K.");
        let full = ui.available_width();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(300.0);
                let te = egui::TextEdit::singleline(&mut self.dict_q).font(ui::ui_font(17.0)).hint_text(ui::hint("Cari: école, ecole, suis, kucing…")).desired_width(300.0).margin(egui::Margin::symmetric(12, 10));
                let r = ui.add(te);
                if self.screen == Screen::Dict && !r.has_focus() && self.modal.is_none() && self.pop.is_none() && ui.ctx().memory(|m| m.focused().is_none()) { r.request_focus(); }
                if let Some(c) = lesson::accent_bar(ui) { self.dict_q.push(c); }
                ui.add_space(6.0);
                let hits = content::search(&self.dict_q);
                let dict = &content::c().dict;
                let idx_of = |e: &content::Entry| dict.iter().position(|x| std::ptr::eq(x, e));
                if self.dict_q.trim().is_empty() {
                    let mut section = |ui: &mut Ui, title: &str, words: Vec<String>, sel: &mut Option<usize>, hist: &mut Vec<String>| {
                        if words.is_empty() { return; }
                        ui::eyebrow(ui, title, pl.ink3);
                        for w in words { if let Some(e) = content::lookup(&w) { if result_row(ui, e, "", *sel == idx_of(e)).clicked() { *sel = idx_of(e); remember(hist, &e.w); } } }
                        ui.add_space(8.0);
                    };
                    let saved: Vec<String> = self.st.dict_saved.iter().rev().take(8).cloned().collect();
                    let hist: Vec<String> = self.st.dict_history.iter().rev().take(10).cloned().collect();
                    section(ui, "Tersimpan", saved, &mut self.dict_sel, &mut self.st.dict_history);
                    section(ui, "Riwayat", hist, &mut self.dict_sel, &mut self.st.dict_history);
                    if self.st.dict_saved.is_empty() && self.st.dict_history.is_empty() {
                        ui::small(ui, &format!("{} entri terkurasi. Pencarian mengabaikan aksen, mengenali bentuk kata (suis → être), dan bisa dari bahasa Indonesia.", dict.len()), pl.ink2);
                    }
                } else if hits.is_empty() {
                    ui::small(ui, "Tidak ditemukan. Coba tanpa kata sandang (le, la, un).", pl.ink2);
                } else {
                    if self.dict_sel.map(|s| !hits.iter().any(|h| idx_of(h.e) == Some(s))).unwrap_or(true) { self.dict_sel = idx_of(hits[0].e); }
                    egui::ScrollArea::vertical().id_salt("hits").max_height(480.0).show(ui, |ui| {
                        for h in &hits { if result_row(ui, h.e, h.why, self.dict_sel == idx_of(h.e)).clicked() { self.dict_sel = idx_of(h.e); remember(&mut self.st.dict_history, &h.e.w); } }
                    });
                }
            });
            ui.add_space(16.0);
            ui.vertical(|ui| {
                ui.set_width(full - 316.0);
                let sel = self.dict_sel.or_else(|| content::c().dict.iter().position(|e| e.w == "bonjour"));
                if let Some(i) = sel { self.entry(ui, &content::c().dict[i]); }
            });
        });
    }

    fn entry(&mut self, ui: &mut Ui, e: &'static content::Entry) {
        let pl = p();
        let sec = |ui: &mut Ui, t: &str| { ui.add_space(14.0); ui::eyebrow(ui, t, pl.ink3); ui.add_space(2.0); };
        let col_w = ui.available_width();
        ui::card().show(ui, |ui| {
            ui.set_max_width(col_w - 40.0);
            ui.set_min_width(col_w - 40.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(&e.w).font(ui::serif(40.0)).strong());
                ui.add_space(8.0);
                if ui::btn(ui, "", Some("speaker"), Kind::Primary, false, true).on_hover_text("Dengar").clicked() { self.say(&e.w, false); }
                if ui::btn(ui, "", Some("slow"), Kind::Normal, false, true).on_hover_text("Dengar pelan").clicked() { self.say(&e.w, true); }
            });
            ui.horizontal(|ui| { ui.label(RichText::new(format!("/{}/", e.ipa)).color(pl.ink2)); ui.label(RichText::new(format!("dibaca “{}”", e.pron)).font(ui::semi(14.0)).color(pl.ink2)); });
            ui.horizontal_wrapped(|ui| {
                ui::chip(ui, &e.pos, pl.surface2, pl.ink2);
                if !e.g.is_empty() { ui::chip(ui, if e.g == "m" { "maskulin" } else { "feminin" }, pl.blue_soft, pl.blue_ink); }
                let reg = match e.reg.as_str() { "soutenu" => "soutenu · resmi", "familier" => "familier · santai", "argot" => "argot · slang", _ => "courant · umum" };
                if e.reg == "familier" { ui::chip(ui, reg, pl.sun_soft, pl.sun_ink); } else { ui::chip(ui, reg, pl.surface2, pl.ink2); }
                ui::chip(ui, &ui::tier_name(e.tier), pl.tier_soft[e.tier as usize], pl.tier_ink[e.tier as usize]);
                ui::chip(ui, &format!("kekerapan {}/5", e.freq), pl.surface2, pl.ink2);
            });
            ui.add_space(4.0);
            let saved = self.st.dict_saved.contains(&e.w);
            if ui::btn(ui, if saved { "Tersimpan" } else { "Simpan untuk ulasan" }, Some("bookmark"), if saved { Kind::Normal } else { Kind::Sun }, false, true).clicked() {
                if saved { self.st.dict_saved.retain(|x| x != &e.w); }
                else { self.st.dict_saved.push(e.w.clone()); self.st.add_card(&e.w, &e.tr[0], &e.pron, ""); self.toast(format!("“{}” masuk ulasan berjarak.", e.w)); }
            }
            if !e.fem.is_empty() || !e.pl.is_empty() {
                let mut v = vec![]; if !e.fem.is_empty() { v.push(format!("Feminin: {}", e.fem)); } if !e.pl.is_empty() { v.push(format!("Jamak: {}", e.pl)); }
                ui::small(ui, &v.join(" · "), pl.ink2);
            }
            sec(ui, "Arti");
            for (i, t) in e.tr.iter().enumerate() { ui.label(RichText::new(format!("{}. {}", i + 1, t)).font(ui::ui_font(17.0))); }
            ui::small(ui, &format!("Sumber: {}", e.src), pl.ink3);
            if !e.def_fr.is_empty() { sec(ui, "Definisi dalam bahasa Prancis"); if let Some(w) = ui::fr_words(ui, &e.def_fr, 17.0, pl.ink) { self.pop = Some(w); } }
            if !e.ex.is_empty() {
                sec(ui, "Contoh");
                for x in &e.ex {
                    ui.horizontal(|ui| {
                        if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { self.say(&x.fr, false); }
                        ui.vertical(|ui| { if let Some(w) = ui::fr_words(ui, &x.fr, 19.0, pl.ink) { self.pop = Some(w); } ui::small(ui, &x.tr, pl.ink2); });
                    });
                }
            }
            if !e.conj.is_empty() {
                sec(ui, "Konjugasi");
                let ti = self.dict_tense.min(e.conj.len() - 1);
                ui.horizontal_wrapped(|ui| { for (i, c) in e.conj.iter().enumerate() { if ui::btn(ui, &c.tense, None, if i == ti { Kind::Primary } else { Kind::Ghost }, false, true).clicked() { self.dict_tense = i; } } });
                egui::Grid::new("conj").num_columns(2).striped(true).spacing(vec2(24.0, 4.0)).show(ui, |ui| {
                    for f in &e.conj[ti].forms {
                        ui.label(RichText::new(f).font(ui::serif(18.0)));
                        if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { self.say(f, false); }
                        ui.end_row();
                    }
                });
            }
            if !e.idi.is_empty() { sec(ui, "Ungkapan"); for x in &e.idi { ui.horizontal(|ui| { ui.label(RichText::new(&x.fr).font(ui::serif(18.0))); ui.label(RichText::new(format!("— {}", x.tr)).color(pl.ink2)); }); } }
            if !e.syn.is_empty() || !e.ant.is_empty() || !e.rel.is_empty() {
                sec(ui, "Kata terkait");
                ui.horizontal_wrapped(|ui| {
                    for s in &e.syn { ui::chip(ui, &format!("sinonim: {s}"), pl.teal_soft, pl.teal); }
                    for s in &e.ant { ui::chip(ui, &format!("antonim: {s}"), pl.red_soft, pl.red_ink); }
                    for s in &e.rel {
                        if ui.add(egui::Button::new(RichText::new(s).font(ui::bold(12.0))).fill(pl.surface2).corner_radius(egui::CornerRadius::same(4))).clicked() {
                            if let Some(x) = content::lookup(s.split(" (").next().unwrap_or(s)) { self.dict_sel = content::c().dict.iter().position(|y| std::ptr::eq(y, x)); remember(&mut self.st.dict_history, &x.w); }
                        }
                    }
                });
            }
            if !e.faux.is_empty() {
                sec(ui, "Awas, kata menipu");
                egui::Frame::new().fill(pl.red_soft).inner_margin(egui::Margin::symmetric(14, 10)).corner_radius(egui::CornerRadius { nw: 0, sw: 0, ne: 6, se: 6 }).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let r = ui.max_rect(); ui.painter().rect_filled(egui::Rect::from_min_size(r.min - vec2(14.0, 10.0), vec2(4.0, r.height() + 20.0)), 0.0, pl.red);
                    ui.label(&e.faux);
                });
            }
            if !e.etym.is_empty() { sec(ui, "Asal kata"); ui.label(&e.etym); }
        });
    }

    /* ══════════ PROGRES ══════════ */
    pub fn progress(&mut self, ui: &mut Ui) {
        let pl = p();
        head(ui, "Progres", "Seberapa lancar kamu sekarang dibanding minggu-minggu lalu. Semua angka diukur dari yang benar-benar kamu kerjakan.");
        let m = self.st.metrics();
        let this = &m.weeks[11]; let last = &m.weeks[10];
        let errs: Vec<(String, u32, u32)> = { let mut v: Vec<_> = m.by_tag.iter().map(|(t, (ok, n))| (t.clone(), n - ok, *n)).filter(|x| x.1 > 0).collect(); v.sort_by(|a, b| b.1.cmp(&a.1)); v.truncate(3); v };
        let acc_now = (this.1.n > 0).then(|| (this.1.ok as f32 / this.1.n as f32 * 100.0).round() as u32);
        let acc_prev = (last.1.n > 0).then(|| (last.1.ok as f32 / last.1.n as f32 * 100.0).round() as u32);
        let due = self.st.due_cards().len();
        let tip = if m.cards == 0 { "Selesaikan pelajaran pertama untuk mulai mengisi angka di sini.".to_string() }
            else if let Some(e) = errs.first().filter(|e| (e.2 - e.1) as f32 / (e.2 as f32) < 0.7) { format!("Ketepatan {} masih {}%. Coba sesi “Latih ini” lima menit.", tag_name(&e.0), ((e.2 - e.1) as f32 / e.2 as f32 * 100.0).round()) }
            else if due > 10 { format!("Ada {due} kartu ulasan menumpuk. Ulasan dulu sebelum pelajaran baru.") }
            else if this.1.speak_sec < 60.0 { "Minggu ini kamu belum banyak bicara. Coba satu dialog atau tiru-ucap.".into() }
            else { "Ritme bagus. Lanjutkan ke unit berikutnya.".into() };
        ui::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui::eyebrow(ui, "Ringkasan minggu ini", pl.blue_ink);
            ui.label(format!("{} menit belajar (minggu lalu {}), {} hari aktif{}.", (this.1.sec / 60.0).round(), (last.1.sec / 60.0).round(), this.2,
                acc_now.map(|a| format!(", ketepatan {a}%{}", acc_prev.map(|b| format!(" (minggu lalu {b}%)")).unwrap_or_default())).unwrap_or_default()));
            ui.label(RichText::new(tip).font(ui::semi(14.5)));
        });
        ui.add_space(12.0);
        let wpm = if self.st.speak.sec > 0.0 { (self.st.speak.words as f64 / (self.st.speak.sec / 60.0)).round() } else { 0.0 };
        ui::cols(ui, 4, |c| {
            let stat = |ui: &mut Ui, label: &str, v: String, unit: &str, how: &str| {
                ui::card().show(ui, |ui| {
                    ui.set_min_height(130.0); ui.set_width(ui.available_width());
                    ui::eyebrow(ui, label, pl.ink2);
                    ui.horizontal(|ui| { ui.label(RichText::new(v).font(ui::display(34.0))); ui.label(RichText::new(unit).color(pl.ink2)); });
                    ui::small(ui, how, pl.ink3);
                });
            };
            stat(&mut c[0], "Kosakata dikuasai", m.mastered.to_string(), "kata", "Kata di tingkat “Dikuasai” pada ulasan berjarak.");
            stat(&mut c[1], "Kosakata aktif", m.active.to_string(), "kata", "Kata berbeda yang kamu ketik, susun, atau ucapkan dengan benar.");
            stat(&mut c[2], "Menit bicara", format!("{}", (self.st.speak.sec / 60.0).round()), "mnt", &format!("Total rekaman. Perkiraan {wpm} kata/menit dari panjang kalimat ÷ durasi rekaman."));
            stat(&mut c[3], "Hari aktif", this.2.to_string(), "/ 7", "Hari dengan minimal satu menit belajar, minggu ini.");
        });
        ui.add_space(12.0);
        ui::cols(ui, 2, |c| {
            ui::card().show(&mut c[0], |ui| {
                ui.set_width(ui.available_width());
                ui::h3(ui, "Perkiraan tier per keterampilan");
                ui::small(ui, "Tier tertinggi yang sudah lulus ujian, diturunkan satu jika ketepatan keterampilan itu di bawah 70%. Ini estimasi, bukan sertifikat.", pl.ink3);
                ui.add_space(6.0);
                for (k, l) in [("baca", "Membaca"), ("dengar", "Mendengar"), ("tulis", "Menulis"), ("bicara", "Berbicara")] {
                    let (acc, n, tier) = self.st.estimate_skill(k, &m);
                    ui.horizontal(|ui| {
                        ui.add_sized(vec2(90.0, 20.0), egui::Label::new(RichText::new(l).font(ui::bold(14.0))));
                        ui.allocate_ui(vec2(ui.available_width() - 170.0, 20.0), |ui| { ui.add_space(5.0); ui::bar(ui, if n > 0 { acc } else { 0.0 }, 10.0, pl.blue, egui::Id::new(("sk", k))); });
                        ui.label(RichText::new(format!("{}{}", if tier >= 0 { ui::tier_name(tier as u8) } else { "Tier 0 · berjalan".into() }, if n > 0 { format!(" · {}%", (acc * 100.0).round()) } else { String::new() })).font(ui::ui_font(13.0)));
                    });
                }
                ui.add_space(4.0);
                ui::small(ui, "Untuk bukti resmi, ikuti ujian DELF atau DALF. Latihan berformat ujian tersedia mulai Tier 3.", pl.ink2);
            });
            ui::card().show(&mut c[1], |ui| {
                ui.set_width(ui.available_width());
                ui::h3(ui, "Kesalahan yang paling sering berulang");
                if errs.is_empty() { ui::muted(ui, "Belum ada pola kesalahan. Angka ini terisi setelah beberapa pelajaran."); }
                for (t, wrong, n) in &errs {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| { ui.label(RichText::new(tag_name(t)).font(ui::bold(14.5))); ui::small(ui, &format!("{wrong} kali salah dari {n} soal · ketepatan {}%", ((n - wrong) as f32 / *n as f32 * 100.0).round()), pl.ink2); });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| { if ui::btn(ui, "Latih ini", Some("target"), Kind::Normal, false, true).clicked() { self.acts.push(Act::Practice(t.clone())); } });
                    });
                    ui.separator();
                }
            });
        });
        ui.add_space(12.0);
        ui::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui::h3(ui, "Tren 12 minggu");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui::small(ui, "ketepatan", pl.ink2); let (r, _) = ui.allocate_exact_size(vec2(14.0, 3.0), egui::Sense::hover()); ui.painter().rect_filled(r, 0.0, pl.blue);
                    ui::small(ui, "menit belajar", pl.ink2); let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover()); ui.painter().rect_filled(r, 2.0, pl.sun);
                });
            });
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 190.0), egui::Sense::hover());
            let (pl_, pr_, pt_, pb_) = (40.0, 44.0, 10.0, 24.0);
            let inner = egui::Rect::from_min_max(rect.min + vec2(pl_, pt_), rect.max - vec2(pr_, pb_));
            let maxm = m.weeks.iter().map(|w| (w.1.sec / 60.0).ceil()).fold(10.0, f64::max);
            let painter = ui.painter();
            for i in 0..=2 { let y = inner.top() + inner.height() * i as f32 / 2.0; painter.line_segment([egui::pos2(inner.left(), y), egui::pos2(inner.right(), y)], egui::Stroke::new(1.0, pl.line)); }
            painter.text(egui::pos2(inner.left() - 6.0, inner.top()), egui::Align2::RIGHT_CENTER, format!("{maxm}"), ui::ui_font(11.0), pl.ink3);
            painter.text(egui::pos2(inner.left() - 6.0, inner.bottom()), egui::Align2::RIGHT_CENTER, "0", ui::ui_font(11.0), pl.ink3);
            painter.text(egui::pos2(inner.right() + 6.0, inner.top()), egui::Align2::LEFT_CENTER, "100%", ui::ui_font(11.0), pl.ink3);
            painter.text(egui::pos2(inner.right() + 6.0, inner.bottom()), egui::Align2::LEFT_CENTER, "0%", ui::ui_font(11.0), pl.ink3);
            let bw = inner.width() / 12.0;
            let mut pts = vec![];
            for (i, (wk, d, _)) in m.weeks.iter().enumerate() {
                let x = inner.left() + i as f32 * bw;
                let hgt = ((d.sec / 60.0) / maxm) as f32 * inner.height();
                if hgt > 0.0 { painter.rect_filled(egui::Rect::from_min_max(egui::pos2(x + bw * 0.22, inner.bottom() - hgt), egui::pos2(x + bw * 0.78, inner.bottom())), 3.0, pl.sun); }
                painter.text(egui::pos2(x + bw / 2.0, rect.bottom() - 8.0), egui::Align2::CENTER_CENTER, wk.format("%d/%m").to_string(), ui::ui_font(11.0), pl.ink3);
                if d.n > 0 { pts.push(egui::pos2(x + bw / 2.0, inner.top() + (1.0 - d.ok as f32 / d.n as f32) * inner.height())); }
            }
            if pts.len() > 1 { painter.add(egui::Shape::line(pts.clone(), egui::Stroke::new(2.5, pl.blue))); }
            for pt in pts { painter.circle(pt, 4.0, pl.surface, egui::Stroke::new(2.0, pl.blue)); }
        });
    }

    /* ══════════ PENGATURAN ══════════ */
    pub fn settings(&mut self, ui: &mut Ui) {
        let pl = p();
        head(ui, "Pengaturan", "");
        let row = |ui: &mut Ui, title: &str, sub: &str, ctrl: &mut dyn FnMut(&mut Ui)| {
            ui.horizontal(|ui| {
                ui.set_min_height(52.0);
                ui.vertical(|ui| { ui.label(RichText::new(title).font(ui::bold(14.5))); if !sub.is_empty() { ui::small(ui, sub, pl.ink2); } });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| ctrl(ui));
            });
            ui.separator();
        };
        fn seg<T: PartialEq + Copy>(ui: &mut Ui, opts: &[(T, &str)], cur: &mut T) {
            for (v, l) in opts.iter().rev() { if ui::btn(ui, l, None, if *cur == *v { Kind::Primary } else { Kind::Ghost }, false, true).clicked() { *cur = *v; } }
        }
        ui::cols(ui, 2, |c| {
            ui::card().show(&mut c[0], |ui| {
                ui.set_width(ui.available_width());
                ui::h3(ui, "Profil");
                row(ui, "Nama panggilan", "Dipakai untuk menyapamu di Beranda.", &mut |ui| { ui.add(egui::TextEdit::singleline(&mut self.st.profile.name).desired_width(180.0).char_limit(24)); });
                row(ui, "Target harian", "Menit belajar per hari.", &mut |ui| seg(ui, &[(5u32, "5"), (10, "10"), (20, "20"), (30, "30")], &mut self.st.profile.goal));
                let mut reason = match self.st.profile.reason.as_str() { "jalan" => 0, "kerja" => 1, "kuliah" => 2, "hobi" => 3, _ => 9 };
                row(ui, "Tujuan belajar", "", &mut |ui| seg(ui, &[(0, "Jalan"), (1, "Kerja"), (2, "Kuliah"), (3, "Hobi")], &mut reason));
                if reason < 9 { self.st.profile.reason = ["jalan", "kerja", "kuliah", "hobi"][reason].into(); }
            });
            ui::card().show(&mut c[1], |ui| {
                ui.set_width(ui.available_width());
                ui::h3(ui, "Suara");
                row(ui, "Bunyi benar / belum tepat", "", &mut |ui| { ui::switch(ui, &mut self.st.settings.sound); });
                let sr = self.st.settings.slow_rate;
                row(ui, "Tempo pelan", &format!("{}% dari tempo normal", (sr * 100.0).round()), &mut |ui| { ui.add(egui::Slider::new(&mut self.st.settings.slow_rate, 0.3..=0.9).show_value(false)); });
                let has = self.au.has_voice();
                let vname = self.au.voice_name().to_string();
                let mut test = false;
                row(ui, "Suara Prancis", &if has { format!("Terpasang: {vname}") } else { "Belum terpasang di Windows.".into() }, &mut |ui| {
                    if has { if ui::btn(ui, "Coba", Some("speaker"), Kind::Normal, false, true).clicked() { test = true; } } else { ui::chip(ui, "Perlu dipasang", pl.sun_soft, pl.sun_ink); }
                });
                if test { self.say("Bonjour ! Je m'appelle Passerelle.", false); }
                if !has {
                    ui::soft(pl.sun_soft).show(ui, |ui| {
                        ui.label(RichText::new("Cara memasang suara Prancis (gratis, sekali saja): ").font(ui::bold(13.5)));
                        ui.label(RichText::new("Pengaturan Windows → Waktu & bahasa → Ucapan → Tambahkan suara → pilih “Français (France)”. Setelah selesai, tutup lalu buka lagi Passerelle. Tanpa suara ini, aplikasi tetap jalan dengan panduan bunyi tertulis.").font(ui::ui_font(13.5)));
                    });
                }
            });
        });
        ui.add_space(12.0);
        ui::cols(ui, 2, |c| {
            ui::card().show(&mut c[0], |ui| {
                ui.set_width(ui.available_width());
                ui::h3(ui, "Tampilan");
                let mut th = match self.st.settings.theme.as_str() { "light" => 1, "dark" => 2, _ => 0 };
                row(ui, "Tema", "", &mut |ui| seg(ui, &[(0, "Ikuti Windows"), (1, "Terang"), (2, "Gelap")], &mut th));
                self.st.settings.theme = ["system", "light", "dark"][th].into();
                let fs = self.st.settings.font_scale;
                row(ui, "Ukuran huruf", &format!("{}%", (fs * 100.0).round()), &mut |ui| { ui.add(egui::Slider::new(&mut self.st.settings.font_scale, 1.0..=1.5).step_by(0.1).show_value(false)); });
                row(ui, "Kurangi animasi", "", &mut |ui| { ui::switch(ui, &mut self.st.settings.reduce_motion); });
                row(ui, "Tampilkan panduan bunyi", "Contoh: bonjour dibaca “bo(n)-ZHUR”.", &mut |ui| { ui::switch(ui, &mut self.st.settings.show_pron); });
            });
            ui::card().show(&mut c[1], |ui| {
                ui.set_width(ui.available_width());
                ui::h3(ui, "Data & privasi");
                ui::small(ui, "Tidak ada akun, server, atau pengumpulan data pemakaian. Progres dan rekaman suara hanya tersimpan di komputer ini.", pl.ink2);
                let mut exp = false; let mut imp = false; let mut wipe = false;
                row(ui, "Cadangkan progres", "Simpan satu berkas .json.", &mut |ui| { if ui::btn(ui, "Ekspor", Some("download"), Kind::Normal, false, true).clicked() { exp = true; } });
                row(ui, "Pulihkan progres", "Menimpa progres sekarang.", &mut |ui| { if ui::btn(ui, "Impor", Some("upload"), Kind::Normal, false, true).clicked() { imp = true; } });
                row(ui, "Hapus semua data saya", "Progres, kartu ulasan, dan rekaman. Tidak bisa dibatalkan.", &mut |ui| { if ui::btn(ui, "Hapus", Some("trash"), Kind::Danger, false, true).clicked() { wipe = true; } });
                if exp {
                    if let Some(path) = rfd::FileDialog::new().set_title("Simpan cadangan progres").set_file_name(format!("passerelle-cadangan-{}.json", store::dkey(store::today()))).add_filter("Cadangan Passerelle", &["json"]).save_file() {
                        if std::fs::write(path, self.st.export()).is_ok() { self.toast("Cadangan tersimpan."); }
                    }
                }
                if imp {
                    if let Some(path) = rfd::FileDialog::new().set_title("Pulihkan cadangan progres").add_filter("Cadangan Passerelle", &["json"]).pick_file() {
                        match std::fs::read_to_string(path).map_err(|e| e.to_string()).and_then(|t| store::State::import(&t)) {
                            Ok(s) => { self.st = s; self.toast("Progres dipulihkan."); }
                            Err(e) => self.toast(e),
                        }
                    }
                }
                if wipe { self.modal = Some(Modal::Confirm(Confirm::WipeAll)); }
            });
        });
        ui.add_space(12.0);
        ui::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui::h3(ui, "Tentang");
            ui::small(ui, "Passerelle 1.0 · artinya “jembatan”. Aplikasi Windows native, dibuat dengan Rust. Kurikulum mengikuti level CEFR (Council of Europe). Huruf: Segoe UI dan Georgia bawaan Windows.", pl.ink2);
            ui::small(ui, "Pintasan: Ctrl+1–5 pindah layar · Ctrl+K kamus cepat · Enter periksa/lanjut · 1–4 pilih jawaban · Spasi dengar ulang · Esc keluar.", pl.ink2);
        });
    }

    /* ══════════ ORIENTASI ══════════ */
    pub fn onboarding(&mut self, ui: &mut Ui) {
        let pl = p();
        let w = 680f32.min(ui.available_width() - 48.0);
        let mut finish = false;
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - w) / 2.0);
            ui.vertical(|ui| {
                ui.set_width(w);
                ui.add_space(56.0);
                let step = self.onb.as_ref().unwrap().step;
                ui.horizontal(|ui| {
                    let sw = (w - 24.0) / 5.0;
                    for i in 0..5u8 { let (r, _) = ui.allocate_exact_size(vec2(sw, 6.0), egui::Sense::hover()); ui.painter().rect_filled(r, 3.0, if i <= step { pl.sun } else { pl.line }); }
                });
                ui.add_space(28.0);
                let choice = |ui: &mut Ui, ic: &str, bg: Color32, fg: Color32, title: &str, sub: &str, width: f32| -> bool {
                    let (r, resp) = ui.allocate_exact_size(vec2(width, 80.0), egui::Sense::click());
                    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                    let down = resp.is_pointer_button_down_on();
                    let face = egui::Rect::from_min_size(r.min + vec2(0.0, if down { 3.0 } else { 0.0 }), vec2(width, 77.0));
                    if !down { ui.painter().rect_filled(face.translate(vec2(0.0, 3.0)), egui::CornerRadius::same(ui::R), pl.line2); }
                    ui.painter().rect(face, egui::CornerRadius::same(ui::R), if resp.hovered() { pl.surface2 } else { pl.surface }, egui::Stroke::new(2.0, pl.line2), egui::StrokeKind::Inside);
                    let icr = egui::Rect::from_min_size(face.min + vec2(16.0, 16.0), vec2(45.0, 45.0));
                    ui.painter().rect_filled(icr, egui::CornerRadius::same(ui::R), bg);
                    ui::icon_at(ui, ic, icr.shrink(11.0), fg);
                    ui.painter().text(face.min + vec2(76.0, 22.0), egui::Align2::LEFT_TOP, title, ui::bold(15.5), pl.ink);
                    ui.painter().text(face.min + vec2(76.0, 44.0), egui::Align2::LEFT_TOP, sub, ui::ui_font(13.0), pl.ink2);
                    resp.clicked()
                };
                let cw = (w - 12.0) / 2.0;
                match step {
                    0 => {
                        ui::icon(ui, "bridge", 72.0, pl.blue);
                        ui.horizontal(|ui| { ui.label(RichText::new("Bienvenue").font(ui::serif(40.0)).strong()); ui.label(RichText::new("di Passerelle").font(ui::display(36.0))); });
                        ui.label(RichText::new("Belajar bahasa Prancis dari nol sampai lancar. Semuanya jalan di komputermu, tanpa internet dan tanpa biaya.").font(ui::ui_font(17.0)).color(pl.ink2));
                        ui.add_space(14.0);
                        let o = self.onb.as_mut().unwrap();
                        let r = ui.add(egui::TextEdit::singleline(&mut o.name).hint_text(ui::hint("Nama panggilanmu (boleh kosong)")).font(ui::ui_font(18.0)).desired_width(w).margin(egui::Margin::symmetric(14, 12)).char_limit(24));
                        if !r.has_focus() && ui.ctx().memory(|m| m.focused().is_none()) { r.request_focus(); }
                        ui.add_space(14.0);
                        if ui::btn(ui, "Mulai", Some("right"), Kind::Primary, true, true).clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.st.profile.name = o.name.trim().to_string(); o.step = 1;
                        }
                    }
                    1 => {
                        ui::h2(ui, "Untuk apa kamu belajar bahasa Prancis?");
                        ui::muted(ui, "Jawabanmu menentukan urutan skenario percakapan.");
                        ui.add_space(12.0);
                        let mut pick = None;
                        egui::Grid::new("why").num_columns(2).spacing(vec2(12.0, 12.0)).show(ui, |ui| {
                            if choice(ui, "plane", pl.blue_soft, pl.blue_ink, "Jalan-jalan", "Kafe, toko, bertanya arah", cw) { pick = Some("jalan"); }
                            if choice(ui, "cap", pl.sun_soft, pl.sun_ink, "Kuliah", "Studi di negara berbahasa Prancis", cw) { pick = Some("kuliah"); }
                            ui.end_row();
                            if choice(ui, "briefcase", pl.teal_soft, pl.teal, "Kerja", "Rapat, email, wawancara", cw) { pick = Some("kerja"); }
                            if choice(ui, "heart", pl.red_soft, pl.red_ink, "Hobi", "Film, musik, teman baru", cw) { pick = Some("hobi"); }
                        });
                        if let Some(r) = pick { self.st.profile.reason = r.into(); self.onb.as_mut().unwrap().step = 2; }
                    }
                    2 => {
                        ui::h2(ui, "Sudah pernah belajar bahasa Prancis?");
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if choice(ui, "seed", pl.sun_soft, pl.sun_ink, "Belum sama sekali", "Mulai dari Tier 0 · Langkah pertama", cw) { self.st.profile.level = "baru".into(); self.onb.as_mut().unwrap().step = 3; }
                            if choice(ui, "bolt", pl.blue_soft, pl.blue_ink, "Sudah tahu dasar", "Tes penempatan 5 menit", cw) { self.st.profile.level = "dasar".into(); self.onb.as_mut().unwrap().step = 3; }
                        });
                    }
                    3 => {
                        ui::h2(ui, "Berapa menit sehari?");
                        ui::muted(ui, "Bisa diubah kapan saja. Sedikit tapi rutin lebih berguna.");
                        ui.add_space(12.0);
                        let mut pick = None;
                        egui::Grid::new("goal").num_columns(2).spacing(vec2(12.0, 12.0)).show(ui, |ui| {
                            if choice(ui, "slow", pl.sun_soft, pl.sun_ink, "5 menit", "Santai", cw) { pick = Some(5); }
                            if choice(ui, "target", pl.blue_soft, pl.blue_ink, "10 menit", "Biasa", cw) { pick = Some(10); }
                            ui.end_row();
                            if choice(ui, "flame", pl.red_soft, pl.red_ink, "20 menit", "Serius", cw) { pick = Some(20); }
                            if choice(ui, "bolt", pl.teal_soft, pl.teal, "30 menit", "Ngebut", cw) { pick = Some(30); }
                        });
                        if let Some(g) = pick { self.st.profile.goal = g; self.onb.as_mut().unwrap().step = 4; }
                    }
                    _ => {
                        ui::h2(ui, "Tes mikrofon dan suara");
                        ui.add_space(10.0);
                        let has_voice = self.au.has_voice();
                        let mut play_take = None; let mut say = false;
                        ui::card().show(ui, |ui| {
                            ui.set_width(w - 40.0);
                            ui.horizontal(|ui| {
                                let o = self.onb.as_mut().unwrap();
                                let on = o.rec.is_some();
                                if ui::btn(ui, if on { "Berhenti" } else { "Rekam" }, Some(if on { "stop" } else { "mic" }), if on { Kind::Danger } else { Kind::Primary }, true, true).clicked() {
                                    if let Some(r) = o.rec.take() {
                                        let t = r.stop();
                                        o.msg = if t.voiced > 0.06 { "Suaramu terdengar jelas. Mikrofon dan pengeras suara siap.".into() } else { "Suaranya pelan sekali. Coba dekatkan mikrofon, atau lanjut saja.".into() };
                                        play_take = Some(t);
                                    } else {
                                        match Recorder::start() { Ok(r) => { o.rec = Some(r); o.msg = "Mikrofon aktif · ucapkan “bonjour”".into(); } Err(_) => o.msg = "Mikrofon tidak terdeteksi. Tidak apa-apa: latihan bicara bisa diganti latihan ketik.".into() }
                                    }
                                }
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| { ui.label(RichText::new("Bonjour !").font(ui::serif(30.0)).strong()); if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { say = true; } });
                                    if let Some(r) = &o.rec { lesson::level_meter(ui, r.level(), r.elapsed()); }
                                    ui.label(RichText::new(if o.msg.is_empty() { "Tekan Rekam dan ucapkan “bonjour”." } else { &o.msg }).color(if o.rec.is_some() { pl.red_ink } else { pl.ink2 }));
                                });
                            });
                        });
                        if let Some(t) = play_take { self.au.play_take(&t); }
                        if say { self.say("Bonjour !", false); }
                        if !has_voice { ui::small(ui, "Suara Prancis belum terpasang di Windows, jadi contoh pengucapan memakai panduan bunyi tertulis. Cara memasangnya ada di Pengaturan.", pl.ink2); }
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            if ui::btn(ui, if self.st.profile.level == "dasar" { "Mulai tes penempatan" } else { "Mulai pelajaran pertama" }, Some("right"), Kind::Primary, true, true).clicked() { finish = true; }
                            if ui::btn(ui, "Lewati tes", None, Kind::Ghost, true, true).clicked() { finish = true; }
                        });
                    }
                }
            });
        });
        if finish {
            self.onb = None;
            self.st.profile.onboarded = true;
            self.st.save();
            if self.st.profile.level == "dasar" { self.acts.push(Act::Exam(0, true)); } else { let s = self.st.next_step(); self.acts.push(Act::Step(s)); }
        }
    }

    /* ══════════ MODAL ══════════ */
    pub fn modals(&mut self, ctx: &egui::Context) {
        let Some(m) = self.modal.clone() else { return };
        let pl = p();
        let mut close = false;
        let resp = egui::Modal::new(egui::Id::new("modal")).frame(egui::Frame::popup(&ctx.style()).fill(pl.surface).corner_radius(egui::CornerRadius::same(ui::R)).inner_margin(egui::Margin::same(24))).show(ctx, |ui| {
            ui.set_width(560.0);
            match &m {
                Modal::Unit(u) => {
                    let ti = u.tier as usize;
                    let pr = self.st.upr(&u.id);
                    let next = self.st.next_step();
                    ui::eyebrow(ui, &format!("{} · {}", ui::tier_name(u.tier), u.theme), pl.tier_ink[ti]);
                    ui.label(RichText::new(&u.title).font(ui::serif(32.0)).strong());
                    let ms = self.st.unit_mastery(u);
                    ui.horizontal(|ui| { ui::small(ui, "Penguasaan:", pl.ink2); ui.label(RichText::new(mastery_name(ms)).font(ui::bold(13.0))); });
                    ui.horizontal(|ui| { let sw = (ui.available_width() - 16.0) / 3.0; for i in 1..=3 { let (r, _) = ui.allocate_exact_size(vec2(sw, 6.0), egui::Sense::hover()); ui.painter().rect_filled(r, 3.0, if ms >= i { pl.tier[ti] } else { pl.surface2 }); } });
                    ui.add_space(10.0);
                    let names = ["Kenali kata (1)", "Kenali kata (2)", "Bentuk kalimat", "Tulis sendiri", "Campuran + ucap"];
                    let first_open = pr.lessons.iter().position(|x| !*x);
                    let mut step_row = |ui: &mut Ui, label: &str, sub: &str, ic: &str, done: bool, avail: bool, nxt: bool| -> bool {
                        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), if avail { egui::Sense::click() } else { egui::Sense::hover() });
                        let resp = if avail { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp };
                        ui.painter().rect(r, egui::CornerRadius::same(ui::R), if resp.hovered() && avail { pl.surface2 } else { pl.surface }, egui::Stroke::new(1.0, pl.line), egui::StrokeKind::Inside);
                        let c = egui::pos2(r.left() + 30.0, r.center().y);
                        let (bg, fg) = if done { (pl.blue, pl.on_blue) } else if nxt { (pl.sun, ui::LIGHT.ink) } else { (pl.surface2, pl.ink2) };
                        ui.painter().circle_filled(c, 18.0, bg);
                        ui::icon_at(ui, if done { "check" } else { ic }, egui::Rect::from_center_size(c, egui::Vec2::splat(18.0)), fg);
                        let a = if avail { 1.0 } else { 0.5 };
                        ui.painter().text(egui::pos2(r.left() + 60.0, r.center().y - 9.0), egui::Align2::LEFT_CENTER, label, ui::bold(14.5), pl.ink.gamma_multiply(a));
                        ui.painter().text(egui::pos2(r.left() + 60.0, r.center().y + 10.0), egui::Align2::LEFT_CENTER, sub, ui::ui_font(12.0), pl.ink2.gamma_multiply(a));
                        ui::icon_at(ui, if avail { "right" } else { "lock" }, egui::Rect::from_center_size(egui::pos2(r.right() - 24.0, r.center().y), egui::Vec2::splat(18.0)), pl.ink3);
                        resp.clicked()
                    };
                    for i in 0..5 {
                        let avail = pr.lessons[i] || first_open == Some(i);
                        if step_row(ui, &format!("Pelajaran {}: {}", i + 1, names[i]), if pr.lessons[i] { "Selesai · bisa diulang" } else { "12–15 soal" }, "book", pr.lessons[i], avail, next == Step::Lesson(u, i)) {
                            self.acts.push(Act::Step(Step::Lesson(u, i))); close = true;
                        }
                    }
                    let all = pr.lessons.iter().all(|x| *x);
                    if step_row(ui, &format!("Dialog: {}", u.dialog.title), &format!("Bersama {}{}", u.dialog.partner, if pr.skipped { " · dilewati" } else { "" }), "chat", pr.dialog, all || pr.dialog, next == Step::Dialog(u)) { self.acts.push(Act::Dialog(u)); close = true; }
                    if step_row(ui, "Ulasan unit", "Naikkan penguasaan kata unit ini", "repeat", pr.review, pr.dialog, next == Step::UnitReview(u)) { self.acts.push(Act::Step(Step::UnitReview(u))); close = true; }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui::btn(ui, "Catatan tata bahasa", Some("note"), Kind::Ghost, false, true).clicked() { self.modal = Some(Modal::Note(u)); }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| { if ui::btn(ui, "Tutup", None, Kind::Normal, false, true).clicked() { close = true; } });
                    });
                }
                Modal::Note(u) => { note_body(ui, u, &mut self.pop); ui.add_space(8.0); ui.with_layout(Layout::right_to_left(Align::Min), |ui| { if ui::btn(ui, "Mengerti", None, Kind::Primary, false, true).clicked() { close = true; } }); }
                Modal::Why(title, body, unit) => {
                    match unit { Some(u) if body.is_empty() => note_body(ui, u, &mut self.pop), _ => { ui::h2(ui, title); ui.add_space(6.0); ui.label(RichText::new(body).font(ui::ui_font(16.0))); } }
                    ui.add_space(12.0);
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| { if ui::btn(ui, "Mengerti", None, Kind::Primary, false, true).clicked() { close = true; } });
                }
                Modal::Confirm(k) => {
                    let (title, body, ok, danger) = match k {
                        Confirm::ExitSession => ("Keluar dari sesi?", "Kemajuan sesi ini tidak disimpan. Menit belajar tetap tercatat.", "Keluar", true),
                        Confirm::SkipExam(_) => ("Ujian lompat?", "Kalau lulus (80%), semua unit di tier ini dianggap selesai dan tier berikutnya terbuka. Kalau belum, tidak ada yang hilang.", "Mulai ujian", false),
                        Confirm::WipeAll => ("Hapus semua data?", "Semua progres, kartu ulasan, pengaturan, dan rekaman suara dihapus permanen dari komputer ini.", "Hapus semua", true),
                        Confirm::ClearRecs => ("Hapus semua rekaman?", "Rekaman suara dihapus permanen dari komputer ini.", "Hapus semua", true),
                    };
                    ui::h2(ui, title); ui.add_space(6.0); ui::muted(ui, body); ui.add_space(18.0);
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui::btn(ui, ok, None, if danger { Kind::Danger } else { Kind::Primary }, false, true).clicked() {
                            close = true;
                            match k {
                                Confirm::ExitSession => { if let Some(s) = self.session.take() { self.st.add_time(s.elapsed()); } }
                                Confirm::SkipExam(t) => self.acts.push(Act::Exam(*t, true)),
                                Confirm::WipeAll => { audio::rec_clear(); store::State::wipe(); self.st = store::State::default(); self.st.v = 1; self.st.profile.goal = 10; self.onb = Some(crate::Onb { step: 0, name: String::new(), rec: None, msg: String::new() }); self.screen = Screen::Home; }
                                Confirm::ClearRecs => { audio::rec_clear(); self.recs.clear(); }
                            }
                        }
                        if ui::btn(ui, "Batal", None, Kind::Ghost, false, true).clicked() { close = true; }
                    });
                }
                Modal::Quick(q) => {
                    let mut q = q.clone();
                    let r = ui.add(egui::TextEdit::singleline(&mut q).hint_text(ui::hint("Cari kata Prancis atau Indonesia…")).font(ui::ui_font(18.0)).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(12, 10)));
                    if !r.has_focus() { r.request_focus(); }
                    if let Some(c) = lesson::accent_bar(ui) { q.push(c); }
                    let hits = content::search(&q);
                    for h in hits.iter().take(8) { if result_row(ui, h.e, h.why, false).clicked() { self.acts.push(Act::Dict(h.e.w.clone())); close = true; } }
                    ui::small(ui, "Enter membuka entri lengkap · Esc menutup", pl.ink3);
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) { if let Some(h) = hits.first() { self.acts.push(Act::Dict(h.e.w.clone())); close = true; } }
                    if !close { self.modal = Some(Modal::Quick(q)); }
                }
            }
        });
        if close || resp.should_close() { if matches!(self.modal, Some(ref x) if std::mem::discriminant(x) == std::mem::discriminant(&m)) || close { self.modal = None; } }
    }
}

fn note_body(ui: &mut Ui, u: &Unit, pop: &mut Option<(String, egui::Pos2)>) {
    let pl = p();
    ui::eyebrow(ui, &format!("Catatan tata bahasa · {}", u.title), pl.ink2);
    ui::h2(ui, &u.note.title);
    ui.add_space(6.0);
    egui::ScrollArea::vertical().max_height(440.0).show(ui, |ui| {
        for b in &u.note.blocks {
            if b.kind == "p" {
                // **tebal** → teks tebal
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for (i, part) in b.text.split("**").enumerate() { ui.label(RichText::new(part).font(if i % 2 == 1 { ui::bold(15.0) } else { ui::ui_font(15.0) })); }
                });
                ui.add_space(4.0);
            } else {
                egui::Grid::new(("note", b.rows.len(), &b.rows[0][0])).striped(true).spacing(vec2(16.0, 6.0)).show(ui, |ui| {
                    for (ri, row) in b.rows.iter().enumerate() {
                        for cell in row {
                            if ri == 0 { ui.label(RichText::new(cell.to_uppercase()).font(ui::bold(11.5)).color(pl.ink2)); }
                            else if let Some(w) = ui::fr_words(ui, cell, 16.0, pl.ink) { *pop = Some(w); }
                        }
                        ui.end_row();
                    }
                });
                ui.add_space(6.0);
            }
        }
    });
}

fn remember(hist: &mut Vec<String>, w: &str) { hist.retain(|x| x != w); hist.push(w.to_string()); if hist.len() > 30 { hist.remove(0); } }

fn result_row(ui: &mut Ui, e: &content::Entry, why: &str, sel: bool) -> egui::Response {
    let pl = p();
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 50.0), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    ui.painter().rect_filled(r, egui::CornerRadius::same(6), if sel { pl.blue_soft } else if resp.hovered() { pl.surface2 } else { Color32::TRANSPARENT });
    let g = ui.painter().text(egui::pos2(r.left() + 12.0, r.top() + 15.0), egui::Align2::LEFT_CENTER, &e.w, ui::serif(17.0), pl.ink);
    if !why.is_empty() { ui.painter().text(egui::pos2(g.right() + 8.0, r.top() + 16.0), egui::Align2::LEFT_CENTER, why, ui::ui_font(11.5), pl.ink3); }
    let mut tr = e.tr.join("; "); if tr.chars().count() > 38 { tr = tr.chars().take(36).collect::<String>() + "…"; }
    ui.painter().text(egui::pos2(r.left() + 12.0, r.top() + 36.0), egui::Align2::LEFT_CENTER, tr, ui::ui_font(12.5), pl.ink2);
    resp
}

/// Simpul unit di Jalur: lingkaran berwarna tier dengan "alas" yang bisa ditekan dan cincin progres.
fn node(ui: &mut Ui, tier: usize, ic: &str, open: bool, current: bool, frac: f32) -> egui::Response {
    let pl = p();
    let s = 78.0;
    let (r, resp) = ui.allocate_exact_size(vec2(s + 16.0, s + 16.0), if open { egui::Sense::click() } else { egui::Sense::hover() });
    let resp = if open { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp };
    let down = open && resp.is_pointer_button_down_on();
    let bob = if current && !ui.ctx().style().animation_time.eq(&0.0) { ui.ctx().request_repaint(); ((ui.input(|i| i.time) * 2.6).sin() as f32) * 3.0 } else { 0.0 };
    let c = r.center() + vec2(0.0, bob - 2.0);
    let (fill, base, fg) = if open { (pl.tier[tier], pl.tier[tier].gamma_multiply(0.62).to_opaque(), Color32::WHITE) } else { (pl.surface2, pl.line2, pl.ink3) };
    let base = if open { Color32::from_rgb((fill.r() as f32 * 0.72) as u8, (fill.g() as f32 * 0.72) as u8, (fill.b() as f32 * 0.72) as u8) } else { base };
    if current { ui.painter().circle_stroke(c, s / 2.0 + 6.0, egui::Stroke::new(4.0, pl.tier_soft[tier])); }
    if open && frac > 0.0 && frac < 1.0 {
        let n = 48; let rr = s / 2.0 + 6.0;
        let pts: Vec<egui::Pos2> = (0..=((n as f32 * frac) as usize).max(1)).map(|i| { let a = -std::f32::consts::FRAC_PI_2 + i as f32 / n as f32 * std::f32::consts::TAU; c + vec2(a.cos(), a.sin()) * rr }).collect();
        ui.painter().add(egui::Shape::line(pts, egui::Stroke::new(4.0, pl.tier[tier])));
    }
    if !down { ui.painter().circle_filled(c + vec2(0.0, 5.0), s / 2.0, base); }
    let cc = c + vec2(0.0, if down { 5.0 } else { 0.0 });
    ui.painter().circle_filled(cc, s / 2.0, fill);
    ui::icon_at(ui, ic, egui::Rect::from_center_size(cc, egui::Vec2::splat(30.0)), fg);
    resp
}
