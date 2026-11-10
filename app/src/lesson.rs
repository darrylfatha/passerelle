//! Penyusun soal dan pemutar sesi (pelajaran, ulasan, ujian, latihan) serta dialog bercabang.
use crate::audio::{self, Audio, Recorder, Take};
use crate::content::{self, norm, Item, Unit};
use crate::store::{Card, State};
use crate::ui::{self, p, Kind, OptState};
use egui::{vec2, Align, Key, Layout, RichText, Ui};
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::time::Instant;

pub struct Cx<'a> {
    pub st: &'a mut State,
    pub au: &'a mut Audio,
    pub pop: &'a mut Option<(String, egui::Pos2)>,
    pub toast: &'a mut Vec<(String, Instant)>,
}
impl Cx<'_> {
    pub fn say(&mut self, t: &str, slow: bool) {
        if !self.au.say(t, slow) {
            self.toast.push(("Suara Prancis belum terpasang di Windows. Ikuti panduan bunyi, atau pasang lewat Pengaturan.".into(), Instant::now()));
        }
    }
}

/* ── Pencocok jawaban ── */
fn clean(s: &str) -> String {
    let t: String = s.to_lowercase().replace('’', "'").chars().map(|c| if ".,!?;:«»\"()…".contains(c) { ' ' } else { c }).collect();
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}
pub struct Check { pub ok: bool, pub note: String, pub tag: &'static str }
pub fn check_typed(input: &str, targets: &[String], tier: u8) -> Check {
    let a = clean(input);
    if a.is_empty() { return Check { ok: false, note: String::new(), tag: "ejaan" }; }
    for t in targets {
        let b = clean(t);
        if a == b { return Check { ok: true, note: String::new(), tag: "ejaan" }; }
        if norm(&a) == norm(&b) {
            return if tier <= 1 { Check { ok: true, note: format!("Perhatikan aksen: {t}"), tag: "aksen" } } else { Check { ok: false, note: String::new(), tag: "aksen" } };
        }
        if b.chars().count() > 8 && content::lev(&norm(&a), &norm(&b)) == 1 {
            return Check { ok: true, note: format!("Ada salah ketik kecil. Yang tepat: {t}"), tag: "ejaan" };
        }
    }
    Check { ok: false, note: String::new(), tag: "ejaan" }
}
fn tokens(fr: &str) -> Vec<String> {
    fr.chars().map(|c| if ".,!?;:«»".contains(c) { ' ' } else { c }).collect::<String>().split_whitespace().map(String::from).collect()
}

/* ── Soal ── */
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QK { PickMeaning, PickFr, Listen, Fill, Match, Arrange, Type, Speak }
impl QK {
    pub fn name(&self) -> &'static str {
        match self { QK::PickMeaning => "pick-meaning", QK::PickFr => "pick-fr", QK::Listen => "listen", QK::Fill => "fill", QK::Match => "match", QK::Arrange => "arrange", QK::Type => "type", QK::Speak => "speak" }
    }
}
#[derive(Clone)]
pub struct Q {
    pub k: QK, pub fr: String, pub tr: String, pub pron: String, pub answer: String, pub options: Vec<String>,
    pub pairs: Vec<Item>, pub tokens: Vec<String>, pub bank: Vec<String>, pub tag: String, pub why: String,
    pub unit: String, pub tier: u8, pub card: Option<String>, pub retry: bool, pub from_speak: bool,
}
impl Q {
    fn new(k: QK) -> Self {
        Q { k, fr: String::new(), tr: String::new(), pron: String::new(), answer: String::new(), options: vec![], pairs: vec![], tokens: vec![], bank: vec![],
            tag: "kosakata".into(), why: String::new(), unit: String::new(), tier: 0, card: None, retry: false, from_speak: false }
    }
}

fn pick<T: Clone>(v: &[T], n: usize) -> Vec<T> { let mut v = v.to_vec(); v.shuffle(&mut rand::thread_rng()); v.truncate(n); v }
fn shuffled<T: Clone>(v: &[T]) -> Vec<T> { let mut v = v.to_vec(); v.shuffle(&mut rand::thread_rng()); v }
fn distract(pool: &[Item], correct: &Item, by_fr: bool, n: usize) -> Vec<String> {
    let key = |x: &Item| if by_fr { x.fr.clone() } else { x.tr.clone() };
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(key(correct));
    let mut out = vec![key(correct)];
    for x in shuffled(pool).iter().chain(shuffled(&content::all_vocab()).iter()) {
        if out.len() > n { break; }
        if seen.insert(key(x)) { out.push(key(x)); }
    }
    shuffled(&out)
}
pub fn q_meaning(v: &Item, pool: &[Item]) -> Q { let mut q = Q::new(QK::PickMeaning); q.fr = v.fr.clone(); q.tr = v.tr.clone(); q.pron = v.pron.clone(); q.answer = v.tr.clone(); q.options = distract(pool, v, false, 3); q }
pub fn q_fr(v: &Item, pool: &[Item]) -> Q { let mut q = Q::new(QK::PickFr); q.fr = v.fr.clone(); q.tr = v.tr.clone(); q.answer = v.fr.clone(); q.options = distract(pool, v, true, 3); q }
pub fn q_listen(v: &Item, pool: &[Item]) -> Q { let mut q = Q::new(QK::Listen); q.fr = v.fr.clone(); q.tr = v.tr.clone(); q.pron = v.pron.clone(); q.answer = v.fr.clone(); q.options = distract(pool, v, true, 3); q }
pub fn q_match(items: &[Item]) -> Q { let mut q = Q::new(QK::Match); q.pairs = items.iter().take(5).cloned().collect(); q }
pub fn q_arrange(s: &Item, pool: &[Item]) -> Q {
    let mut q = Q::new(QK::Arrange);
    q.fr = s.fr.clone(); q.tr = s.tr.clone(); q.tokens = tokens(&s.fr); q.tag = "urutan kata".into();
    let ins: HashSet<String> = q.tokens.iter().map(|t| t.to_lowercase()).collect();
    let extra: Vec<String> = shuffled(pool).into_iter().filter(|v| !v.fr.contains(' ') && !ins.contains(&v.fr.to_lowercase())).take(2).map(|v| v.fr).collect();
    q.bank = shuffled(&[q.tokens.clone(), extra].concat());
    q
}
pub fn q_type(s: &Item) -> Q { let mut q = Q::new(QK::Type); q.fr = s.fr.clone(); q.tr = s.tr.clone(); q.answer = s.fr.clone(); q.tag = "ejaan".into(); q }
pub fn q_fill(f: &content::Fill) -> Q { let mut q = Q::new(QK::Fill); q.fr = f.fr.clone(); q.answer = f.answer.clone(); q.options = shuffled(&f.options); q.why = f.why.clone(); q.tag = f.tag.clone(); q }
pub fn q_speak(s: &Item) -> Q { let mut q = Q::new(QK::Speak); q.fr = s.fr.clone(); q.tr = s.tr.clone(); q.pron = s.pron.clone(); q.tag = "pelafalan".into(); q }

fn stamp(mut qs: Vec<Q>, u: &Unit) -> Vec<Q> { for q in &mut qs { q.unit = u.id.clone(); q.tier = u.tier; } qs }

pub fn build_lesson(u: &Unit, li: usize) -> Vec<Q> {
    let v = &u.vocab; let a = &v[..6.min(v.len())]; let b = &v[6.min(v.len())..]; let s = &u.sentences; let f = &u.fills;
    let mut qs: Vec<Q> = match li {
        0 => [vec![q_match(&pick(a, 5))], pick(a, 3).iter().map(|x| q_meaning(x, a)).collect(), pick(a, 3).iter().map(|x| q_fr(x, a)).collect(),
              pick(a, 2).iter().map(|x| q_listen(x, a)).collect(), vec![q_type(&a[0]), q_speak(&a[0]), q_meaning(&a[a.len() - 1], v)]].concat(),
        1 => [vec![q_match(&pick(b, 5))], pick(b, 3).iter().map(|x| q_meaning(x, b)).collect(), pick(b, 3).iter().map(|x| q_fr(x, b)).collect(),
              pick(b, 2).iter().map(|x| q_listen(x, b)).collect(), vec![q_arrange(&s[0], v), q_speak(&pick(b, 1)[0]), q_fr(&pick(a, 1)[0], v)]].concat(),
        2 => [pick(s, 4).iter().map(|x| q_arrange(x, v)).collect::<Vec<_>>(), pick(f, 3).iter().map(q_fill).collect(), pick(s, 2).iter().map(|x| q_listen(x, s)).collect(),
              pick(s, 2).iter().map(|x| q_fr(x, s)).collect(), vec![q_speak(&pick(s, 1)[0])]].concat(),
        3 => [pick(s, 4).iter().map(q_type).collect::<Vec<_>>(), pick(v, 2).iter().map(q_type).collect(), f.iter().map(q_fill).collect(),
              pick(s, 2).iter().map(|x| q_arrange(x, v)).collect(), vec![q_speak(&pick(s, 1)[0])]].concat(),
        _ => [vec![q_match(&pick(v, 5))], pick(s, 2).iter().map(|x| q_arrange(x, v)).collect(), pick(s, 2).iter().map(q_type).collect(),
              pick(f, 2).iter().map(q_fill).collect(), pick(v, 2).iter().map(|x| q_listen(x, v)).collect(), pick(s, 2).iter().map(q_speak).collect(),
              vec![q_meaning(&pick(v, 1)[0], v)]].concat(),
    };
    if li >= 2 { let first = qs.remove(0); qs.shuffle(&mut rand::thread_rng()); qs.insert(0, first); }
    stamp(qs, u)
}
pub fn build_unit_review(u: &Unit) -> Vec<Q> {
    let v = &u.vocab; let s = &u.sentences;
    stamp([pick(v, 4).iter().map(|x| q_meaning(x, v)).collect::<Vec<_>>(), pick(v, 3).iter().map(q_type).collect(), pick(v, 2).iter().map(|x| q_listen(x, v)).collect(),
        pick(s, 2).iter().map(|x| q_arrange(x, v)).collect(), vec![q_fill(&pick(&u.fills, 1)[0]), q_speak(&pick(s, 1)[0])]].concat(), u)
}
pub fn build_daily(cards: &[Card]) -> Vec<Q> {
    let pool = content::all_vocab();
    cards.iter().take(15).enumerate().map(|(i, c)| {
        let v = Item { fr: c.fr.clone(), tr: c.tr.clone(), pron: c.pron.clone() };
        let mut q = match (c.level as usize + i) % 3 { 0 => q_meaning(&v, &pool), 1 => q_type(&v), _ => q_listen(&v, &pool) };
        q.card = Some(c.fr.clone()); q.unit = c.unit.clone(); q.tier = content::unit(&c.unit).map(|u| u.tier).unwrap_or(0);
        q
    }).collect()
}
pub fn build_exam(tier: u8) -> Vec<Q> {
    let us = content::units_of_tier(tier);
    let v: Vec<Item> = us.iter().flat_map(|u| u.vocab.clone()).collect();
    let s: Vec<Item> = us.iter().flat_map(|u| u.sentences.clone()).collect();
    let f: Vec<content::Fill> = us.iter().flat_map(|u| u.fills.clone()).collect();
    let mut qs = [pick(&v, 3).iter().map(|x| q_meaning(x, &v)).collect::<Vec<_>>(), pick(&s, 2).iter().map(|x| q_fr(x, &s)).collect(),
        pick(&v, 2).iter().map(|x| q_listen(x, &v)).collect(), pick(&s, 2).iter().map(|x| q_listen(x, &s)).collect(), pick(&s, 3).iter().map(q_type).collect(),
        pick(&s, 2).iter().map(|x| q_arrange(x, &v)).collect(), pick(&f, 3).iter().map(q_fill).collect(), pick(&s, 1).iter().map(q_speak).collect()].concat();
    qs.shuffle(&mut rand::thread_rng());
    for q in &mut qs { q.tier = tier; }
    qs
}
pub fn build_practice(tag: &str, st: &State) -> Vec<Q> {
    let us: Vec<&Unit> = content::c().units.iter().filter(|u| st.unit_unlocked(u) || st.unit_done(&u.id)).collect();
    let mut qs: Vec<Q> = Vec::new();
    for u in &us { for f in &u.fills { if f.tag == tag { let mut q = q_fill(f); q.unit = u.id.clone(); q.tier = u.tier; qs.push(q); } } }
    qs.shuffle(&mut rand::thread_rng()); qs.truncate(8);
    let v: Vec<Item> = us.iter().flat_map(|u| u.vocab.clone()).collect();
    let s: Vec<Item> = us.iter().flat_map(|u| u.sentences.clone()).collect();
    while qs.len() < 8 && !v.is_empty() {
        qs.push(match tag { "urutan kata" => q_arrange(&pick(&s, 1)[0], &v), "ejaan" | "aksen" => q_type(&pick(&[s.clone(), v.clone()].concat(), 1)[0]), _ => q_meaning(&pick(&v, 1)[0], &v) });
    }
    qs
}

/* ── Sesi ── */
#[derive(Clone)]
pub enum SKind { Lesson(&'static Unit, usize), UnitReview(&'static Unit), Daily, Exam(u8, bool), Practice }

enum Phase { Ask, Checked { ok: bool, title: String, detail: String, note: String }, Finished { acc: u32, sec: f64, new_cards: usize, headline: String, sub: String, passed: Option<bool> } }

pub enum Out { None, Close, Continue, ConfirmExit, Why(String, String, Option<&'static Unit>) }

pub struct Session {
    pub title: String, pub kind: SKind,
    queue: Vec<Q>, idx: usize, total: usize, first_try: usize, done: usize, missed: HashSet<usize>,
    start: Instant, phase: Phase,
    sel: Option<String>, typed: String, arranged: Vec<usize>,
    m_left: Vec<usize>, m_right: Vec<usize>, m_sel_l: Option<usize>, m_sel_r: Option<usize>, m_done: HashSet<usize>, m_mistakes: u32, m_flash: Option<(usize, usize, Instant)>,
    rec: Option<Recorder>, take: Option<Take>, rec_err: String, auto_said: bool,
}

impl Session {
    pub fn new(title: String, kind: SKind, qs: Vec<Q>) -> Self {
        let total = qs.len();
        let mut s = Session { title, kind, queue: qs, idx: 0, total, first_try: 0, done: 0, missed: HashSet::new(), start: Instant::now(), phase: Phase::Ask,
            sel: None, typed: String::new(), arranged: vec![], m_left: vec![], m_right: vec![], m_sel_l: None, m_sel_r: None, m_done: HashSet::new(), m_mistakes: 0, m_flash: None,
            rec: None, take: None, rec_err: String::new(), auto_said: false };
        s.reset_q();
        s
    }
    pub fn started(&self) -> bool { self.done > 0 }
    pub fn elapsed(&self) -> f64 { self.start.elapsed().as_secs_f64() }
    fn q(&self) -> &Q { &self.queue[self.idx] }
    fn reset_q(&mut self) {
        self.sel = None; self.typed.clear(); self.arranged.clear(); self.m_sel_l = None; self.m_sel_r = None; self.m_done.clear(); self.m_mistakes = 0; self.m_flash = None;
        self.rec = None; self.take = None; self.rec_err.clear(); self.auto_said = false;
        if self.idx < self.queue.len() {
            let n = self.queue[self.idx].pairs.len();
            let mut l: Vec<usize> = (0..n).collect(); l.shuffle(&mut rand::thread_rng());
            let mut r: Vec<usize> = (0..n).collect(); r.shuffle(&mut rand::thread_rng());
            self.m_left = l; self.m_right = r;
        }
        self.phase = Phase::Ask;
    }
    fn ready(&self) -> bool {
        match self.q().k {
            QK::PickMeaning | QK::PickFr | QK::Listen | QK::Fill => self.sel.is_some(),
            QK::Arrange => !self.arranged.is_empty(),
            QK::Type => !self.typed.trim().is_empty(),
            _ => false,
        }
    }

    fn grade(&mut self, cx: &mut Cx, ok: bool, title: String, detail: String, note: String, tag: Option<&str>) {
        let q = self.q().clone();
        let first = !self.missed.contains(&self.idx) && !q.retry;
        cx.st.record(q.k.name(), tag.unwrap_or(&q.tag), ok);
        if let Some(c) = &q.card { cx.st.grade_card(c, ok); }
        else if matches!(q.k, QK::PickMeaning | QK::Type | QK::Listen) && cx.st.has_card(&q.fr) { cx.st.grade_card(&q.fr, ok); }
        if ok {
            self.done += 1;
            if first { self.first_try += 1; }
            if matches!(q.k, QK::Type | QK::Arrange | QK::Speak) { cx.st.mark_active(&q.fr); }
            cx.au.ok();
        } else {
            // Tanpa nyawa: soal yang salah diulang di akhir sesi.
            let mut r = q.clone(); r.retry = true; self.queue.push(r);
            self.missed.insert(self.idx);
            cx.au.bad();
        }
        self.phase = Phase::Checked { ok, title, detail, note };
    }

    fn check(&mut self, cx: &mut Cx) {
        if !self.ready() { return; }
        let q = self.q().clone();
        match q.k {
            QK::PickMeaning | QK::PickFr | QK::Listen | QK::Fill => {
                let ok = self.sel.as_deref() == Some(q.answer.as_str());
                let detail = match q.k {
                    QK::PickMeaning => format!("{} = {}", q.fr, q.answer),
                    QK::Fill => q.fr.replace("___", &q.answer),
                    _ => if q.tr.is_empty() { q.answer.clone() } else { format!("{} = {}", q.answer, q.tr) },
                };
                self.grade(cx, ok, "Benar".into(), if ok { detail } else { format!("Jawaban: {detail}") }, String::new(), None);
            }
            QK::Arrange => {
                let got: Vec<String> = self.arranged.iter().map(|&i| q.bank[i].clone()).collect();
                let ok = clean(&got.join(" ")) == clean(&q.tokens.join(" "));
                self.grade(cx, ok, "Benar".into(), if ok { q.fr.clone() } else { format!("Jawaban: {}", q.fr) }, String::new(), None);
            }
            QK::Type => {
                let r = check_typed(&self.typed, &[q.answer.clone()], q.tier);
                self.grade(cx, r.ok, "Benar".into(), if r.ok { if r.note.is_empty() { q.answer.clone() } else { String::new() } } else { format!("Jawaban: {}", q.answer) }, r.note, Some(r.tag));
            }
            _ => {}
        }
    }

    fn next(&mut self, cx: &mut Cx) {
        self.idx += 1;
        if self.idx >= self.queue.len() { self.finish(cx); } else { self.reset_q(); }
    }

    fn finish(&mut self, cx: &mut Cx) {
        let sec = self.elapsed();
        cx.st.add_time(sec);
        let acc = ((self.first_try as f64 / self.total.max(1) as f64) * 100.0).round() as u32;
        let mut new_cards = 0; let mut passed = None;
        let (headline, sub) = match self.kind.clone() {
            SKind::Lesson(u, i) => {
                cx.st.up(&u.id).lessons[i] = true;
                cx.st.day_mut().lessons += 1;
                let add: Vec<Item> = if i == 0 { u.vocab[..6].to_vec() } else if i == 1 { u.vocab[6..].to_vec() } else { vec![] };
                for v in add { if cx.st.add_card(&v.fr, &v.tr, &v.pron, &u.id) { new_cards += 1; } }
                if i == 4 { ("Lima pelajaran beres".to_string(), "Tinggal dialog dan ulasan unit.".to_string()) } else { ("Pelajaran selesai".into(), format!("Pelajaran {} dari 5 sudah terbuka.", i + 2)) }
            }
            SKind::UnitReview(u) => { cx.st.up(&u.id).review = true; ("Unit selesai".into(), format!("“{}” tuntas. Unit berikutnya terbuka.", u.title)) }
            SKind::Exam(t, skipping) => {
                let ok = acc >= 80; passed = Some(ok);
                if ok {
                    cx.st.progress.tiers.insert(t);
                    for u in content::units_of_tier(t) {
                        if !cx.st.unit_done(&u.id) { let pr = cx.st.up(&u.id); pr.lessons = [true; 5]; pr.dialog = true; pr.review = true; pr.skipped = true; }
                        for v in &u.vocab { cx.st.add_card(&v.fr, &v.tr, &v.pron, &u.id); }
                    }
                    let nxt = content::c().tiers.get(t as usize + 1).map(|x| format!("Tier {} · {} terbuka.", x.n, x.label)).unwrap_or_default();
                    (format!("Lulus Tier {t}"), format!("Skor {acc}%. {nxt}"))
                } else {
                    ("Belum lulus kali ini".into(), format!("Skor {acc}%, butuh 80%. {}", if skipping { "Lanjutkan dari unit pertama, kamu akan cepat sampai." } else { "Ulangi ulasan unit lalu coba lagi." }))
                }
            }
            SKind::Daily => { new_cards = self.total; ("Ulasan selesai".into(), "Kartu berikutnya muncul sesuai jadwal.".into()) }
            SKind::Practice => { new_cards = self.total; ("Latihan selesai".into(), "Kesalahan ini akan muncul lebih jarang.".into()) }
        };
        cx.au.done();
        self.phase = Phase::Finished { acc, sec, new_cards, headline, sub, passed };
    }

    /* ── Tampilan ── */
    pub fn ui(&mut self, ui: &mut Ui, cx: &mut Cx) -> Out {
        let pl = p();
        let mut out = Out::None;
        let typing = ui.ctx().memory(|m| m.focused().is_some());
        let (enter, esc, num, space) = ui.input(|i| (
            i.key_pressed(Key::Enter) && !i.modifiers.shift, i.key_pressed(Key::Escape),
            [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5].iter().position(|k| i.key_pressed(*k)), i.key_pressed(Key::Space)));

        // Bilah atas
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            if ui::icon_btn(ui, "x", "Keluar (Esc)", Kind::Ghost).clicked() { out = if self.started() && !matches!(self.phase, Phase::Finished { .. }) { Out::ConfirmExit } else { Out::Close }; }
            let frac = if matches!(self.phase, Phase::Finished { .. }) { 1.0 } else { self.done as f32 / self.total.max(1) as f32 };
            ui.add_space(8.0);
            let w = ui.available_width() - 200.0;
            ui.allocate_ui(vec2(w, 36.0), |ui| { ui.add_space(11.0); ui::bar(ui, frac, 14.0, pl.sun, egui::Id::new("sesi-bar")); });
            ui.add_space(12.0);
            ui.label(RichText::new(&self.title).font(ui::ui_font(13.0)).color(pl.ink2));
        });
        if esc && !matches!(self.phase, Phase::Finished { .. }) { out = if self.started() { Out::ConfirmExit } else { Out::Close }; }

        if let Phase::Finished { acc, sec, new_cards, headline, sub, passed } = &self.phase {
            let (acc, sec, new_cards, headline, sub, passed) = (*acc, *sec, *new_cards, headline.clone(), sub.clone(), *passed);
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                let size = 120.0;
                let (r, _) = ui.allocate_exact_size(egui::Vec2::splat(size + 6.0), egui::Sense::hover());
                let t = ui.ctx().animate_bool_with_time(egui::Id::new("stamp"), true, 0.35);
                let rr = egui::Rect::from_center_size(r.center(), egui::Vec2::splat(size * (1.4 - 0.4 * t)));
                let (bg, base, fg) = if passed == Some(false) { (pl.surface2, pl.line2, pl.ink2) } else { (pl.sun, pl.sun_press, ui::LIGHT.ink) };
                ui.painter().circle_filled(rr.center() + vec2(0.0, 6.0), rr.width() / 2.0, base.gamma_multiply(t));
                ui.painter().circle_filled(rr.center(), rr.width() / 2.0, bg.gamma_multiply(t));
                ui::icon_at(ui, if passed == Some(false) { "repeat" } else if matches!(self.kind, SKind::Exam(..)) { "trophy" } else { "star" }, egui::Rect::from_center_size(rr.center(), egui::Vec2::splat(52.0)), fg.gamma_multiply(t));
                ui.add_space(18.0);
                ui::h1(ui, &headline);
                ui::muted(ui, &sub);
                ui.add_space(20.0);
                let shown = |id: &str, v: f32| ui.ctx().animate_value_with_time(egui::Id::new(id), v, 0.8);
                let a = shown("s-acc", acc as f32); let c = shown("s-cards", new_cards as f32);
                ui.allocate_ui(vec2(560.0, 110.0), |ui| {
                    ui::cols(ui, 3, |cols| {
                        let stat = |ui: &mut Ui, big: String, lbl: &str| { ui::card().show(ui, |ui| { ui.vertical_centered(|ui| { ui.label(RichText::new(big).font(ui::display(28.0))); ui::small(ui, lbl, p().ink2); }); }); };
                        stat(&mut cols[0], format!("{}%", a.round()), "benar di percobaan pertama");
                        stat(&mut cols[1], ui::fmt_min(sec), "waktu belajar");
                        stat(&mut cols[2], format!("{}", c.round()), if matches!(self.kind, SKind::Daily | SKind::Practice) { "kartu diulas" } else { "kata baru di ulasan" });
                    });
                });
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 330.0).max(0.0) / 2.0);
                    if ui::btn(ui, "Kembali", None, Kind::Ghost, true, true).clicked() { out = Out::Close; }
                    if ui::btn(ui, "Lanjutkan", Some("right"), Kind::Primary, true, true).clicked() || enter { out = Out::Continue; }
                });
            });
            return out;
        }

        let foot_h = 104.0;
        let body_h = ui.available_height() - foot_h;
        // Isi soal
        ui.allocate_ui(vec2(ui.available_width(), body_h), |ui| {
            egui::ScrollArea::vertical().id_salt(("q", self.idx)).auto_shrink([false, false]).show(ui, |ui| {
                let w = 760f32.min(ui.available_width() - 48.0);
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - w) / 2.0).max(24.0));
                    ui.vertical(|ui| { ui.set_width(w); ui.add_space(24.0); self.body(ui, cx, w, num, space && !typing); });
                });
            });
        });

        // Bilah bawah
        let pl2 = p();
        let (fill, stroke) = match &self.phase { Phase::Checked { ok: true, .. } => (pl2.blue_soft, pl2.blue), Phase::Checked { ok: false, .. } => (pl2.red_soft, pl2.red), _ => (pl2.surface, pl2.line) };
        let r = ui.available_rect_before_wrap();
        ui.painter().rect_filled(r, 0.0, fill);
        ui.painter().line_segment([r.left_top(), r.right_top()], egui::Stroke::new(2.0, stroke));
        ui.horizontal(|ui| {
            ui.set_height(foot_h);
            let w = 960f32.min(ui.available_width() - 48.0);
            ui.add_space(((ui.available_width() - w) / 2.0).max(24.0));
            ui.allocate_ui_with_layout(vec2(w, foot_h), Layout::left_to_right(Align::Center), |ui| {
                match &self.phase {
                    Phase::Ask => {
                        if ui::btn(ui, "Kenapa?", Some("help"), Kind::Ghost, false, true).clicked() { out = self.why(false); }
                        if self.q().k == QK::Speak && ui::btn(ui, "Mikrofon bermasalah? Ketik saja", Some("keyboard"), Kind::Ghost, false, true).clicked() { self.to_typing(); }
                        if !matches!(self.q().k, QK::Match | QK::Speak) {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let ready = self.ready();
                                if ui::btn(ui, "Periksa", None, Kind::Primary, true, ready).clicked() || (enter && ready) { self.check(cx); }
                            });
                        }
                    }
                    Phase::Checked { ok, title, detail, note } => {
                        let (ok, title, detail, note) = (*ok, title.clone(), detail.clone(), note.clone());
                        let (c, ic) = if ok { (pl2.blue, "check") } else { (pl2.red, "x") };
                        let (br, _) = ui.allocate_exact_size(egui::Vec2::splat(52.0), egui::Sense::hover());
                        let t = ui.ctx().animate_bool_with_time(egui::Id::new(("badge", self.idx)), true, 0.22);
                        ui.painter().circle_filled(br.center(), 26.0 * (0.6 + 0.4 * t), c);
                        ui::icon_at(ui, ic, egui::Rect::from_center_size(br.center(), egui::Vec2::splat(28.0)), if ok { pl2.on_blue } else { egui::Color32::WHITE });
                        ui.vertical(|ui| {
                            ui.add_space(4.0);
                            ui.label(RichText::new(if ok { title } else { "Belum tepat".into() }).font(ui::display(21.0)).color(if ok { pl2.blue_ink } else { pl2.red_ink }));
                            if !detail.is_empty() { ui.label(RichText::new(detail).font(ui::serif(16.0)).color(pl2.ink)); }
                            if !note.is_empty() { ui.label(RichText::new(note).font(ui::semi(13.5)).color(pl2.ink)); }
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui::btn(ui, "Lanjut", None, if ok { Kind::Primary } else { Kind::Danger }, true, true).clicked() || enter { self.next(cx); }
                            if ui::btn(ui, "Kenapa?", Some("help"), Kind::Ghost, false, true).clicked() { out = self.why(true); }
                        });
                    }
                    Phase::Finished { .. } => {}
                }
            });
        });
        if self.rec.is_some() { ui.ctx().request_repaint(); }
        out
    }

    fn why(&self, checked: bool) -> Out {
        let q = self.q();
        let u = content::unit(&q.unit);
        if checked && !q.why.is_empty() { return Out::Why("Kenapa begitu?".into(), q.why.clone(), None); }
        match u { Some(u) => Out::Why(u.note.title.clone(), String::new(), Some(u)), None => Out::Why("Tips".into(), "Klik kata Prancis mana pun untuk melihat artinya.".into(), None) }
    }
    fn to_typing(&mut self) {
        let mut q = self.q().clone();
        q.k = QK::Type; q.answer = q.fr.clone(); q.tag = "ejaan".into(); q.from_speak = true;
        self.queue[self.idx] = q;
        self.reset_q();
    }

    fn options(&mut self, ui: &mut Ui, cx: &mut Cx, w: f32, french: bool, num: Option<usize>) {
        let q = self.q().clone();
        let checked = !matches!(self.phase, Phase::Ask);
        let one = q.options.iter().any(|o| o.chars().count() > 24);
        let cols = if one { 1 } else { 2 };
        let cw = (w - 12.0 * (cols as f32 - 1.0)) / cols as f32;
        if let (Some(n), false) = (num, checked) { if let Some(o) = q.options.get(n) { self.sel = Some(o.clone()); cx.au.tap(); } }
        egui::Grid::new(("opts", self.idx)).num_columns(cols).spacing(vec2(12.0, 12.0)).show(ui, |ui| {
            for (i, o) in q.options.iter().enumerate() {
                let st = if checked {
                    if *o == q.answer { OptState::Right } else if self.sel.as_deref() == Some(o) { OptState::Wrong } else { OptState::Normal }
                } else if self.sel.as_deref() == Some(o) { OptState::Selected } else { OptState::Normal };
                if ui::opt(ui, Some(i + 1), o, french, st, cw, !checked).clicked() { self.sel = Some(o.clone()); cx.au.tap(); }
                if (i + 1) % cols == 0 { ui.end_row(); }
            }
        });
    }

    fn prompt_listen(&self, ui: &mut Ui, cx: &mut Cx, text: &str) {
        if ui::btn(ui, "", Some("speaker"), Kind::Primary, true, true).on_hover_text("Dengar (Spasi)").clicked() { cx.say(text, false); }
        if ui::btn(ui, "", Some("slow"), Kind::Normal, true, true).on_hover_text("Dengar pelan").clicked() { cx.say(text, true); }
    }

    fn body(&mut self, ui: &mut Ui, cx: &mut Cx, w: f32, num: Option<usize>, space: bool) {
        let pl = p();
        let q = self.q().clone();
        let checked = !matches!(self.phase, Phase::Ask);
        let title = |ui: &mut Ui, s: &str| { ui.label(RichText::new(s).font(ui::display(26.0))); ui.add_space(10.0); };
        let prompt_card = |ui: &mut Ui, s: &str| { ui::soft(p().surface2).show(ui, |ui| { ui.set_width(w - 32.0); ui.label(RichText::new(s).font(ui::semi(19.0))); }); ui.add_space(12.0); };
        if space && !q.fr.is_empty() && matches!(q.k, QK::Listen | QK::Speak | QK::PickMeaning) { cx.say(&q.fr, false); }
        match q.k {
            QK::PickMeaning => {
                title(ui, "Apa artinya?");
                ui.horizontal(|ui| {
                    self.prompt_listen(ui, cx, &q.fr);
                    ui.add_space(8.0);
                    ui.vertical(|ui| { ui::fr_label(ui, &q.fr, 30.0); if cx.st.settings.show_pron && !q.pron.is_empty() { ui::small(ui, &format!("dibaca “{}”", q.pron), pl.ink2); } });
                });
                ui.add_space(16.0);
                self.options(ui, cx, w, false, num);
            }
            QK::PickFr => { title(ui, "Pilih bahasa Prancisnya"); prompt_card(ui, &q.tr); self.options(ui, cx, w, true, num); }
            QK::Listen => {
                let voice = cx.au.has_voice();
                if voice && !self.auto_said { self.auto_said = true; cx.say(&q.fr, false); }
                title(ui, if voice { "Dengar, lalu pilih yang kamu dengar" } else { "Baca panduan bunyinya, lalu pilih tulisannya" });
                ui.horizontal(|ui| {
                    if voice { self.prompt_listen(ui, cx, &q.fr); ui.add_space(8.0); }
                    let shown = if voice { "Putar lagi: Spasi".to_string() } else { format!("“{}”", if q.pron.is_empty() { guess_pron(&q.fr) } else { q.pron.clone() }) };
                    ui::soft(pl.surface2).show(ui, |ui| { ui.label(RichText::new(shown).font(ui::display(22.0)).extra_letter_spacing(0.6)); });
                });
                ui.add_space(16.0);
                self.options(ui, cx, w, true, num);
            }
            QK::Fill => {
                title(ui, "Lengkapi kalimatnya");
                let shown = q.fr.replace("___", &format!(" {} ", self.sel.clone().unwrap_or_else(|| "_____".into())));
                ui.label(RichText::new(shown).font(ui::serif(28.0)).color(pl.ink));
                ui.add_space(16.0);
                self.options(ui, cx, w, true, num);
            }
            QK::Match => {
                title(ui, "Pasangkan katanya");
                let n = q.pairs.len();
                let cw = (w - 16.0) / 2.0;
                let flash = self.m_flash.filter(|f| f.2.elapsed().as_secs_f32() < 0.4);
                if flash.is_some() { ui.ctx().request_repaint(); } else { self.m_flash = None; }
                let mut clicked_l = None; let mut clicked_r = None;
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(cw);
                        for (row, &i) in self.m_left.iter().enumerate() {
                            let st = if self.m_done.contains(&i) { OptState::Done } else if flash.map(|f| f.0 == i).unwrap_or(false) { OptState::Wrong } else if self.m_sel_l == Some(i) { OptState::Selected } else { OptState::Normal };
                            if ui::opt(ui, Some(row + 1), &q.pairs[i].fr, true, st, cw, !checked && st != OptState::Done).clicked() { clicked_l = Some(i); }
                            ui.add_space(4.0);
                        }
                    });
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        ui.set_width(cw);
                        for &i in &self.m_right {
                            let st = if self.m_done.contains(&i) { OptState::Done } else if flash.map(|f| f.1 == i).unwrap_or(false) { OptState::Wrong } else if self.m_sel_r == Some(i) { OptState::Selected } else { OptState::Normal };
                            if ui::opt(ui, None, &q.pairs[i].tr, false, st, cw, !checked && st != OptState::Done).clicked() { clicked_r = Some(i); }
                            ui.add_space(4.0);
                        }
                    });
                });
                if let Some(nk) = num { if !checked { if let Some(&i) = self.m_left.get(nk) { if !self.m_done.contains(&i) { clicked_l = Some(i); } } } }
                if let Some(i) = clicked_l { self.m_sel_l = Some(i); cx.au.tap(); }
                if let Some(i) = clicked_r { self.m_sel_r = Some(i); cx.au.tap(); }
                if let (Some(l), Some(r)) = (self.m_sel_l, self.m_sel_r) {
                    if l == r { self.m_done.insert(l); cx.au.tap(); if cx.au.has_voice() { let fr = q.pairs[l].fr.clone(); cx.au.say(&fr, false); } }
                    else { self.m_mistakes += 1; self.m_flash = Some((l, r, Instant::now())); cx.au.bad(); cx.st.record("match", "kosakata", false); }
                    self.m_sel_l = None; self.m_sel_r = None;
                    if self.m_done.len() == n && !checked {
                        let m = self.m_mistakes;
                        self.grade(cx, true, if m > 0 { "Semua pasangan cocok".into() } else { "Sempurna".into() }, if m > 0 { format!("{m} kali salah pasang") } else { "Tanpa salah pasang".into() }, String::new(), None);
                    }
                }
            }
            QK::Arrange => {
                title(ui, "Susun kalimatnya");
                prompt_card(ui, &q.tr);
                let mut remove = None; let mut add = None;
                let tile = |ui: &mut Ui, s: &str, ghost: bool| -> bool {
                    let g = ui.fonts(|f| f.layout_no_wrap(s.to_string(), ui::serif(19.0), p().ink));
                    let size = vec2(g.size().x + 28.0, 44.0);
                    let (r, resp) = ui.allocate_exact_size(size + vec2(0.0, 3.0), egui::Sense::click());
                    if ghost { ui.painter().rect_filled(r.shrink2(vec2(0.0, 1.5)), egui::CornerRadius::same(ui::R), p().surface2); return false; }
                    let down = resp.is_pointer_button_down_on();
                    let face = egui::Rect::from_min_size(r.min + vec2(0.0, if down { 3.0 } else { 0.0 }), size);
                    if !down { ui.painter().rect_filled(face.translate(vec2(0.0, 3.0)), egui::CornerRadius::same(ui::R), p().line2); }
                    ui.painter().rect(face, egui::CornerRadius::same(ui::R), p().surface, egui::Stroke::new(2.0, p().line2), egui::StrokeKind::Inside);
                    ui.painter().galley(face.center() - g.size() / 2.0, g, p().ink);
                    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
                };
                // jawaban
                let (ar, _) = ui.allocate_exact_size(vec2(w, 0.0), egui::Sense::hover());
                ui.horizontal_wrapped(|ui| {
                    ui.set_min_height(64.0);
                    for (pos, &bi) in self.arranged.iter().enumerate() { if tile(ui, &q.bank[bi], false) && !checked { remove = Some(pos); } }
                });
                let y = ui.cursor().top() + 4.0;
                ui.painter().line_segment([egui::pos2(ar.left(), y), egui::pos2(ar.left() + w, y)], egui::Stroke::new(2.0, pl.line2));
                ui.add_space(24.0);
                ui.horizontal_wrapped(|ui| {
                    for (bi, s) in q.bank.iter().enumerate() { let used = self.arranged.contains(&bi); if tile(ui, s, used) && !used && !checked { add = Some(bi); } }
                });
                if let Some(pos) = remove { self.arranged.remove(pos); cx.au.tap(); }
                if let Some(bi) = add { self.arranged.push(bi); cx.au.tap(); }
            }
            QK::Type => {
                title(ui, if q.from_speak { "Ketik kalimat ini dalam bahasa Prancis" } else { "Tulis dalam bahasa Prancis" });
                prompt_card(ui, &q.tr);
                let te = egui::TextEdit::multiline(&mut self.typed).font(ui::serif(22.0)).desired_rows(3).desired_width(w).hint_text(ui::hint("Ketik dalam bahasa Prancis…")).interactive(!checked);
                let r = ui.add(te);
                if !checked && !r.has_focus() && self.typed.is_empty() { r.request_focus(); }
                if r.has_focus() && ui.input(|i| i.key_pressed(Key::Enter)) { self.typed = self.typed.trim_end_matches('\n').to_string(); }
                ui.add_space(6.0);
                if let Some(c) = accent_bar(ui) { self.typed.push(c); r.request_focus(); }
                ui::small(ui, if q.tier <= 1 { "Di Tier 0–1, aksen yang terlewat hanya diberi catatan." } else { "Aksen dihitung." }, pl.ink3);
            }
            QK::Speak => {
                title(ui, "Ucapkan kalimat ini");
                ui.horizontal(|ui| {
                    self.prompt_listen(ui, cx, &q.fr);
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        if let Some((wd, pos)) = ui::fr_words(ui, &q.fr, 28.0, pl.ink) { *cx.pop = Some((wd, pos)); }
                        if !q.tr.is_empty() { ui::muted(ui, &q.tr); }
                        if cx.st.settings.show_pron && !q.pron.is_empty() { ui::small(ui, &format!("dibaca “{}”", q.pron), pl.ink2); }
                    });
                });
                ui.add_space(20.0);
                let mut confirm = false;
                ui.horizontal(|ui| {
                    let recording = self.rec.is_some();
                    let (r, resp) = ui.allocate_exact_size(egui::Vec2::splat(92.0), egui::Sense::click());
                    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(if recording { "Berhenti merekam" } else { "Mulai merekam" });
                    let (c, base) = if recording { (pl.red, pl.red_press) } else { (pl.blue, pl.blue_press) };
                    let down = resp.is_pointer_button_down_on();
                    let cc = r.center() + vec2(0.0, if down { 4.0 } else { 0.0 });
                    if recording { let t = ui.input(|i| i.time) as f32; ui.painter().circle_filled(r.center(), 44.0 + 4.0 * (t * 5.0).sin().abs(), pl.red_soft); }
                    if !down { ui.painter().circle_filled(r.center() + vec2(0.0, 5.0), 42.0, base); }
                    ui.painter().circle_filled(cc, 42.0, c);
                    ui::icon_at(ui, if recording { "stop" } else { "mic" }, egui::Rect::from_center_size(cc, egui::Vec2::splat(36.0)), if recording { egui::Color32::WHITE } else { pl.on_blue });
                    if resp.clicked() && !checked {
                        if let Some(rec) = self.rec.take() { self.take = Some(rec.stop()); }
                        else { match Recorder::start() { Ok(r) => { self.rec = Some(r); self.take = None; self.rec_err.clear(); } Err(e) => self.rec_err = format!("Mikrofon tidak bisa dipakai ({e}). Pakai tombol “Ketik saja”.") } }
                    }
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        if let Some(rec) = &self.rec {
                            ui.label(RichText::new("●  Mikrofon aktif · merekam").font(ui::bold(14.0)).color(pl.red_ink));
                            level_meter(ui, rec.level(), rec.elapsed());
                        } else if let Some(t) = &self.take {
                            if t.voiced < 0.06 { ui.label(RichText::new("Suaramu hampir tidak terdengar. Coba dekatkan mikrofon, atau lanjut kalau yakin.").font(ui::semi(14.0)).color(pl.red_ink)); }
                            else { ui::muted(ui, &format!("Rekaman {:.1} detik. Bandingkan dengan contoh, lalu nilai sendiri.", t.sec)); }
                        } else if !self.rec_err.is_empty() { ui.label(RichText::new(&self.rec_err).color(pl.red_ink)); }
                        else { ui::muted(ui, "Tekan tombol mikrofon, ucapkan kalimatnya, lalu tekan lagi."); }
                    });
                });
                if let Some(t) = self.take.clone() {
                    if !checked {
                        ui.add_space(12.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui::btn(ui, "Putar rekamanku", Some("play"), Kind::Normal, false, true).clicked() { cx.au.play_take(&t); }
                            if ui::btn(ui, "Dengar contoh", Some("speaker"), Kind::Normal, false, true).clicked() { cx.say(&q.fr, false); }
                            if ui::btn(ui, "Saya sudah mengucapkannya dengan benar", Some("check"), Kind::Primary, false, true).clicked() { confirm = true; }
                        });
                        if ui.input(|i| i.key_pressed(Key::Enter)) { confirm = true; }
                    }
                }
                if confirm {
                    let t = self.take.clone().unwrap();
                    save_take(cx, &q.fr, &t, "tiru");
                    self.grade(cx, true, "Terdengar benar".into(), q.fr.clone(), String::new(), None);
                }
            }
        }
    }
}

pub fn save_take(cx: &mut Cx, text: &str, t: &Take, kind: &str) {
    let words = text.split_whitespace().filter(|w| w.chars().any(|c| c.is_alphabetic())).count() as u32;
    cx.st.speak.sec += t.sec as f64; cx.st.speak.words += words; cx.st.speak.takes += 1;
    cx.st.day_mut().speak_sec += t.sec as f64;
    audio::rec_save(text, kind, t);
}

pub fn level_meter(ui: &mut Ui, level: f32, t: f32) {
    let pl = p();
    let (r, _) = ui.allocate_exact_size(vec2(220.0, 36.0), egui::Sense::hover());
    for i in 0..24 {
        let ph = (t * 9.0 + i as f32 * 0.7).sin().abs();
        let hgt = (6.0 + level * 260.0 * (0.4 + 0.6 * ph)).min(34.0);
        let x = r.left() + i as f32 * 9.0;
        ui.painter().rect_filled(egui::Rect::from_center_size(egui::pos2(x + 2.0, r.center().y), vec2(4.0, hgt)), egui::CornerRadius::same(2), pl.red);
    }
}

/// Bilah huruf beraksen; mengembalikan huruf yang diklik.
pub fn accent_bar(ui: &mut Ui) -> Option<char> {
    let pl = p();
    let mut out = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for c in ['é', 'è', 'ê', 'à', 'â', 'ç', 'ù', 'û', 'ô', 'î', 'ï', 'ë', 'œ', '«', '»'] {
            let (r, resp) = ui.allocate_exact_size(vec2(38.0, 38.0), egui::Sense::click());
            let down = resp.is_pointer_button_down_on();
            let face = egui::Rect::from_min_size(r.min + vec2(0.0, if down { 2.0 } else { 0.0 }), vec2(38.0, 36.0));
            if !down { ui.painter().rect_filled(face.translate(vec2(0.0, 2.0)), egui::CornerRadius::same(6), pl.line2); }
            ui.painter().rect(face, egui::CornerRadius::same(6), if resp.hovered() { pl.surface2 } else { pl.surface }, egui::Stroke::new(1.0, pl.line2), egui::StrokeKind::Inside);
            ui.painter().text(face.center(), egui::Align2::CENTER_CENTER, c, ui::serif(18.0), pl.ink);
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() { out = Some(c); }
        }
    });
    out
}

/// Panduan bunyi sederhana untuk kalimat yang tidak punya panduan tertulis.
pub fn guess_pron(fr: &str) -> String {
    let mut s = fr.to_lowercase().replace(['?', '!', '.', ','], "");
    for (a, b) in [("eau", "o"), ("au", "o"), ("ou", "u"), ("ai", "é"), ("oi", "wa"), ("qu", "k"), ("ch", "sy"), ("ç", "s"), ("j", "zh")] { s = s.replace(a, b); }
    s.split_whitespace().map(|w| {
        let mut w = w.to_string();
        for (a, b) in [("on", "o(n)"), ("an", "a(n)"), ("en", "a(n)")] { if w.ends_with(a) { w = format!("{}{}", &w[..w.len() - a.len()], b); } }
        if w.len() > 2 && (w.ends_with('t') || w.ends_with('s') || w.ends_with('d') || w.ends_with('x')) { w.pop(); }
        if w.len() > 2 && w.ends_with('e') && !w.ends_with("ée") { w.pop(); }
        w
    }).collect::<Vec<_>>().join(" ")
}

/* ── Dialog bercabang ── */
pub enum Line { Npc(String, String), Me(String, String), Hint(String) }
pub struct DialogRun {
    pub unit: &'static Unit,
    node: String, lines: Vec<Line>, turns: u32, hints: u32,
    pub speak_mode: bool, pub show_tr: bool, ended: bool, start: Instant,
    pending: Option<(String, Instant)>, speaking: Option<usize>, rec: Option<Recorder>, take: Option<Take>, said: bool, scroll: bool,
}
pub enum DOut { None, Close, Done, Again, Skip }

impl DialogRun {
    pub fn new(unit: &'static Unit) -> Self {
        let d = &unit.dialog;
        let n = &d.nodes[&d.start];
        DialogRun { unit, node: d.start.clone(), lines: vec![Line::Npc(n.say.clone(), n.tr.clone())], turns: 0, hints: 0, speak_mode: false, show_tr: false, ended: false,
            start: Instant::now(), pending: None, speaking: None, rec: None, take: None, said: false, scroll: true }
    }
    fn choose(&mut self, cx: &mut Cx, i: usize) {
        let d = &self.unit.dialog;
        let r = d.nodes[&self.node].replies[i].clone();
        self.turns += 1;
        self.lines.push(Line::Me(r.fr.clone(), r.tr.clone()));
        self.speaking = None; self.take = None; self.rec = None;
        if !r.hint.is_empty() {
            self.hints += 1; cx.au.bad();
            self.lines.push(Line::Hint(r.hint.clone()));
            cx.st.record("dialog", "kosakata", false);
            if r.next.is_empty() { self.scroll = true; return; }
        } else { cx.st.record("dialog", "kosakata", true); cx.st.mark_active(&r.fr); }
        self.pending = Some((r.next.clone(), Instant::now()));
        self.scroll = true;
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut Cx) -> DOut {
        let pl = p();
        let d = &self.unit.dialog;
        let mut out = DOut::None;
        if !self.said { self.said = true; if let Some(Line::Npc(s, _)) = self.lines.first() { let s = s.clone(); cx.au.say(&s, false); } }
        if let Some((next, t)) = self.pending.clone() {
            if t.elapsed().as_secs_f32() > 0.45 {
                self.pending = None;
                self.node = next;
                let n = &d.nodes[&self.node];
                self.lines.push(Line::Npc(n.say.clone(), n.tr.clone()));
                cx.au.say(&n.say, false);
                if n.end && !self.ended {
                    self.ended = true;
                    let pr = cx.st.up(&self.unit.id); pr.dialog = true;
                    cx.st.progress.dialog_skip_streak = 0;
                    cx.st.add_time(self.start.elapsed().as_secs_f64());
                    cx.au.done();
                }
                self.scroll = true;
            } else { ui.ctx().request_repaint(); }
        }
        let num = ui.input(|i| [Key::Num1, Key::Num2, Key::Num3, Key::Num4].iter().position(|k| i.key_pressed(*k)));
        if ui.input(|i| i.key_pressed(Key::Escape)) { out = DOut::Close; }

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            if ui::icon_btn(ui, "x", "Keluar (Esc)", Kind::Ghost).clicked() { out = DOut::Close; }
            ui.vertical(|ui| {
                ui::eyebrow(ui, &format!("Dialog · {}", self.unit.title), pl.ink2);
                ui.label(RichText::new(format!("{} — bersama {}, {}", d.title, d.partner, d.role)).font(ui::bold(16.0)));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(24.0);
                let skips = cx.st.progress.dialog_skip_streak;
                if !self.ended {
                    let r = ui::btn(ui, if skips >= 2 { "Tidak bisa dilewati lagi" } else { "Lewati" }, None, Kind::Ghost, false, skips < 2);
                    if r.on_hover_text("Dialog tidak bisa dilewati lebih dari dua kali berturut-turut.").clicked() { out = DOut::Skip; }
                }
                if ui::btn(ui, if self.show_tr { "Sembunyikan terjemahan" } else { "Tampilkan terjemahan" }, None, Kind::Ghost, false, true).clicked() { self.show_tr = !self.show_tr; }
                if ui::btn(ui, "Ucapkan", Some("mic"), if self.speak_mode { Kind::Primary } else { Kind::Normal }, false, true).clicked() { self.speak_mode = true; }
                if ui::btn(ui, "Pilih", None, if self.speak_mode { Kind::Normal } else { Kind::Primary }, false, true).clicked() { self.speak_mode = false; self.speaking = None; }
            });
        });
        ui.add_space(8.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
            let w = 760f32.min(ui.available_width() - 48.0);
            ui.horizontal(|ui| {
                ui.add_space(((ui.available_width() - w) / 2.0).max(24.0));
                ui.vertical(|ui| {
                    ui.set_width(w);
                    for l in &self.lines {
                        match l {
                            Line::Npc(s, t) => {
                                ui.horizontal(|ui| {
                                    ui::card().inner_margin(egui::Margin::symmetric(16, 12)).show(ui, |ui| {
                                        ui.set_max_width(w * 0.78);
                                        ui::small(ui, &d.partner, pl.ink3);
                                        ui.horizontal(|ui| {
                                            if let Some(c) = ui::fr_words(ui, s, 20.0, pl.ink) { *cx.pop = Some(c); }
                                            if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { cx.say(s, false); }
                                        });
                                        if self.show_tr { ui::small(ui, t, pl.ink2); }
                                    });
                                });
                            }
                            Line::Me(s, t) => {
                                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                    egui::Frame::new().fill(pl.blue).corner_radius(egui::CornerRadius::same(ui::R)).inner_margin(egui::Margin::symmetric(16, 12)).show(ui, |ui| {
                                        ui.set_max_width(w * 0.78);
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(s).font(ui::serif(20.0)).color(pl.on_blue));
                                            if self.show_tr { ui.label(RichText::new(t).font(ui::ui_font(12.5)).color(pl.on_blue.gamma_multiply(0.8))); }
                                        });
                                    });
                                });
                            }
                            Line::Hint(h) => {
                                ui.vertical_centered(|ui| { ui::soft(pl.sun_soft).show(ui, |ui| { ui.label(RichText::new(h).font(ui::semi(14.0)).color(pl.sun_ink)); }); });
                            }
                        }
                        ui.add_space(6.0);
                    }
                    ui.add_space(12.0);
                    if self.ended && self.pending.is_none() {
                        ui::card().show(ui, |ui| {
                            ui.set_width(w - 40.0);
                            ui.vertical_centered(|ui| {
                                ui::h2(ui, "Dialog selesai");
                                ui::muted(ui, &format!("{} balasan{}.", self.turns, if self.hints > 0 { format!(", {} di antaranya kurang pas dan sudah diperbaiki", self.hints) } else { ", semuanya pas".into() }));
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.add_space((ui.available_width() - 420.0).max(0.0) / 2.0);
                                    if ui::btn(ui, "Ulangi dengan pilihan lain", Some("repeat"), Kind::Normal, false, true).clicked() { out = DOut::Again; }
                                    if ui::btn(ui, "Lanjutkan", Some("right"), Kind::Primary, false, true).clicked() || ui.input(|i| i.key_pressed(Key::Enter)) { out = DOut::Done; }
                                });
                            });
                        });
                    } else if self.pending.is_none() {
                        let n = &d.nodes[&self.node];
                        if let Some(si) = self.speaking {
                            let r = n.replies[si].clone();
                            ui::card().show(ui, |ui| {
                                ui.set_width(w - 40.0);
                                ui::eyebrow(ui, "Ucapkan balasanmu", pl.ink2);
                                ui.horizontal(|ui| { ui::fr_label(ui, &r.fr, 22.0); if ui::icon_btn(ui, "speaker", "Dengar", Kind::Ghost).clicked() { cx.say(&r.fr, false); } });
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    let rec_on = self.rec.is_some();
                                    if ui::btn(ui, if rec_on { "Berhenti" } else { "Rekam" }, Some(if rec_on { "stop" } else { "mic" }), if rec_on { Kind::Danger } else { Kind::Primary }, true, true).clicked() {
                                        if let Some(rc) = self.rec.take() { let t = rc.stop(); save_take(cx, &r.fr, &t, "dialog"); self.take = Some(t); }
                                        else { match Recorder::start() { Ok(rc) => { self.rec = Some(rc); } Err(_) => cx.toast.push(("Mikrofon tidak bisa dipakai. Pilih balasan dengan klik saja.".into(), Instant::now())) } }
                                    }
                                    if let Some(rc) = &self.rec { ui.label(RichText::new("●  merekam").font(ui::bold(13.0)).color(pl.red_ink)); level_meter(ui, rc.level(), rc.elapsed()); ui.ctx().request_repaint(); }
                                    if let Some(t) = self.take.clone() {
                                        if ui::btn(ui, "Putar", Some("play"), Kind::Normal, false, true).clicked() { cx.au.play_take(&t); }
                                        if ui::btn(ui, "Saya sudah mengucapkannya dengan benar", Some("check"), Kind::Primary, false, true).clicked() { self.choose(cx, si); }
                                    }
                                });
                                if ui.link("Pilih balasan lain").clicked() { self.speaking = None; self.rec = None; self.take = None; }
                            });
                        } else {
                            ui::eyebrow(ui, if self.speak_mode { "Pilih balasan, lalu ucapkan" } else { "Pilih balasanmu" }, pl.ink2);
                            let mut chosen = num.filter(|&i| i < n.replies.len());
                            for (i, r) in n.replies.iter().enumerate() {
                                let label = if self.show_tr { format!("{}\n{}", r.fr, r.tr) } else { r.fr.clone() };
                                if ui::opt(ui, Some(i + 1), &label, true, OptState::Normal, w, true).clicked() { chosen = Some(i); }
                                ui.add_space(2.0);
                            }
                            if let Some(i) = chosen { if self.speak_mode { self.speaking = Some(i); } else { self.choose(cx, i); } }
                        }
                    }
                    ui.add_space(40.0);
                });
            });
        });
        out
    }
}
