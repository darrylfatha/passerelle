//! Status aplikasi: profil, pengaturan, progres, kartu ulasan berjarak, rangkaian hari, dan metrik.
//! Disimpan sebagai JSON di %APPDATA%\Passerelle\state.json. Tidak pernah dikirim ke mana pun.
use crate::content::{self, norm, Unit};
use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

const DAY_MS: i64 = 86_400_000;
pub fn now_ms() -> i64 { Local::now().timestamp_millis() }
pub fn dkey(d: NaiveDate) -> String { d.format("%Y-%m-%d").to_string() }
pub fn today() -> NaiveDate { Local::now().date_naive() }
pub fn week_start(d: NaiveDate) -> NaiveDate { d - Duration::days(d.weekday().num_days_from_monday() as i64) }

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Profile { pub name: String, pub reason: String, pub goal: u32, pub onboarded: bool, pub level: String }
impl Default for Profile { fn default() -> Self { Self { name: String::new(), reason: String::new(), goal: 10, onboarded: false, level: String::new() } } }

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct Settings { pub theme: String, pub font_scale: f32, pub reduce_motion: bool, pub sound: bool, pub slow_rate: f32, pub show_pron: bool }
impl Default for Settings { fn default() -> Self { Self { theme: "system".into(), font_scale: 1.0, reduce_motion: false, sound: true, slow_rate: 0.6, show_pron: true } } }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct UnitProg { pub lessons: [bool; 5], pub dialog: bool, pub review: bool, pub skipped: bool }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Progress { pub units: HashMap<String, UnitProg>, pub tiers: BTreeSet<u8>, pub dialog_skip_streak: u32 }

#[derive(Serialize, Deserialize, Clone)]
pub struct Card { pub fr: String, pub tr: String, pub pron: String, pub unit: String, pub level: u8, pub due: i64, pub right: u32, pub wrong: u32, pub last: i64 }

#[derive(Serialize, Deserialize, Clone)]
pub struct Attempt { pub t: i64, pub kind: String, pub tag: String, pub ok: bool }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Day { pub sec: f64, pub speak_sec: f64, pub lessons: u32, pub ok: u32, pub n: u32 }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Speak { pub sec: f64, pub words: u32, pub takes: u32 }

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct State {
    pub v: u32,
    pub profile: Profile,
    pub settings: Settings,
    pub progress: Progress,
    pub cards: HashMap<String, Card>,
    pub attempts: Vec<Attempt>,
    pub days: BTreeMap<String, Day>,
    pub active: HashSet<String>,
    pub dict_history: Vec<String>,
    pub dict_saved: Vec<String>,
    pub speak: Speak,
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let d = base.join("Passerelle");
    let _ = std::fs::create_dir_all(&d);
    d
}
fn state_path() -> PathBuf { data_dir().join("state.json") }

#[derive(Clone, Copy)]
pub enum Step { Lesson(&'static Unit, usize), Dialog(&'static Unit), UnitReview(&'static Unit), Exam(u8), None }
impl PartialEq for Step {
    fn eq(&self, o: &Self) -> bool {
        match (self, o) {
            (Step::Lesson(a, i), Step::Lesson(b, j)) => a.id == b.id && i == j,
            (Step::Dialog(a), Step::Dialog(b)) | (Step::UnitReview(a), Step::UnitReview(b)) => a.id == b.id,
            (Step::Exam(a), Step::Exam(b)) => a == b,
            (Step::None, Step::None) => true,
            _ => false,
        }
    }
}

pub struct Metrics {
    pub mastered: usize, pub cards: usize, pub active: usize,
    pub by_tag: BTreeMap<String, (u32, u32)>, pub by_skill: BTreeMap<&'static str, (u32, u32)>,
    pub weeks: Vec<(NaiveDate, Day, u32)>, // (awal minggu, total, hari aktif)
}

pub fn skill_of(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "pick-meaning" | "pick-fr" | "match" | "fill" => "baca",
        "listen" => "dengar",
        "arrange" | "type" => "tulis",
        "speak" | "dialog" => "bicara",
        _ => return None,
    })
}

impl State {
    pub fn load() -> Self {
        let mut s: State = std::fs::read_to_string(state_path()).ok().and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default();
        s.v = 1;
        if s.profile.goal == 0 { s.profile.goal = 10; }
        if s.settings.font_scale < 1.0 { s.settings.font_scale = 1.0; }
        s
    }
    pub fn save(&self) {
        if let Ok(t) = serde_json::to_string(self) {
            let tmp = state_path().with_extension("tmp");
            if std::fs::write(&tmp, t).is_ok() { let _ = std::fs::rename(&tmp, state_path()); }
        }
    }
    pub fn export(&self) -> String { serde_json::to_string_pretty(self).unwrap_or_default() }
    pub fn import(txt: &str) -> Result<State, String> {
        let s: State = serde_json::from_str(txt.trim_start_matches('\u{feff}')).map_err(|_| "Berkas ini bukan cadangan Passerelle.".to_string())?;
        if s.v != 1 { return Err("Versi cadangan tidak dikenal.".into()); }
        Ok(s)
    }
    pub fn wipe() { let _ = std::fs::remove_dir_all(data_dir()); }

    /* ── Waktu ── */
    pub fn today_day(&self) -> Day { self.days.get(&dkey(today())).cloned().unwrap_or_default() }
    pub fn add_time(&mut self, sec: f64) { self.days.entry(dkey(today())).or_default().sec += sec; }
    pub fn day_mut(&mut self) -> &mut Day { self.days.entry(dkey(today())).or_default() }

    /// Rangkaian hari, dengan dua hari jeda per minggu (Senin–Minggu).
    pub fn streak(&self) -> (u32, BTreeSet<String>) {
        let active = |d: NaiveDate| self.days.get(&dkey(d)).map(|x| x.sec >= 60.0).unwrap_or(false);
        let mut used: HashMap<NaiveDate, u32> = HashMap::new();
        let mut freezes = BTreeSet::new();
        let mut n = 0;
        let mut earliest = String::from("9999");
        let mut d = today();
        if !active(d) { d -= Duration::days(1); }
        for _ in 0..400 {
            if active(d) { n += 1; earliest = dkey(d); }
            else {
                let wk = week_start(d);
                let u = used.entry(wk).or_insert(0);
                if *u < 2 && n > 0 { *u += 1; freezes.insert(dkey(d)); } else { break; }
            }
            d -= Duration::days(1);
        }
        freezes.retain(|k| *k >= earliest);
        (n, freezes)
    }

    /* ── Kurikulum ── */
    pub fn up(&mut self, id: &str) -> &mut UnitProg { self.progress.units.entry(id.to_string()).or_default() }
    pub fn upr(&self, id: &str) -> UnitProg { self.progress.units.get(id).cloned().unwrap_or_default() }
    pub fn unit_done(&self, id: &str) -> bool { let u = self.upr(id); u.lessons.iter().all(|x| *x) && u.dialog && u.review }
    pub fn unit_steps(&self, id: &str) -> usize { let u = self.upr(id); u.lessons.iter().filter(|x| **x).count() + u.dialog as usize + u.review as usize }
    pub fn tier_passed(&self, t: u8) -> bool { self.progress.tiers.contains(&t) }
    pub fn tier_unlocked(&self, t: u8) -> bool { t == 0 || self.tier_passed(t - 1) }
    pub fn unit_unlocked(&self, u: &Unit) -> bool {
        if !self.tier_unlocked(u.tier) { return false; }
        let list = content::units_of_tier(u.tier);
        let i = list.iter().position(|x| x.id == u.id).unwrap_or(0);
        i == 0 || self.unit_done(&list[i - 1].id)
    }
    pub fn tier_exam_ready(&self, t: u8) -> bool { let l = content::units_of_tier(t); !l.is_empty() && l.iter().all(|u| self.unit_done(&u.id)) }

    pub fn next_step(&self) -> Step {
        for t in &content::c().tiers {
            if !self.tier_unlocked(t.n) { break; }
            let units = content::units_of_tier(t.n);
            if units.is_empty() { return Step::None; }
            for u in units {
                if !self.unit_unlocked(u) { continue; }
                let p = self.upr(&u.id);
                if let Some(i) = p.lessons.iter().position(|x| !*x) { return Step::Lesson(u, i); }
                if !p.dialog { return Step::Dialog(u); }
                if !p.review { return Step::UnitReview(u); }
            }
            if !self.tier_passed(t.n) { return Step::Exam(t.n); }
        }
        Step::None
    }

    /* ── Ulasan berjarak ── */
    pub fn add_card(&mut self, fr: &str, tr: &str, pron: &str, unit: &str) -> bool {
        let k = norm(fr);
        if self.cards.contains_key(&k) { return false; }
        self.cards.insert(k, Card { fr: fr.into(), tr: tr.into(), pron: pron.into(), unit: unit.into(), level: 1, due: now_ms() + DAY_MS, right: 0, wrong: 0, last: now_ms() });
        true
    }
    pub fn has_card(&self, fr: &str) -> bool { self.cards.contains_key(&norm(fr)) }
    pub fn grade_card(&mut self, fr: &str, ok: bool) {
        const IV: [i64; 7] = [0, 1, 2, 4, 8, 16, 32];
        if let Some(c) = self.cards.get_mut(&norm(fr)) {
            if ok { c.level = (c.level + 1).min(6); c.right += 1; c.due = now_ms() + IV[c.level as usize] * DAY_MS; }
            else { c.level = c.level.saturating_sub(2).max(1); c.wrong += 1; c.due = now_ms() + 10 * 60_000; }
            c.last = now_ms();
        }
    }
    pub fn due_cards(&self) -> Vec<Card> {
        let mut v: Vec<Card> = self.cards.values().filter(|c| c.due <= now_ms()).cloned().collect();
        v.sort_by_key(|c| c.due);
        v
    }
    /// 0 belum, 1 Dikenal, 2 Terlatih, 3 Dikuasai
    pub fn unit_mastery(&self, u: &Unit) -> u8 {
        let sum: u32 = u.vocab.iter().filter_map(|v| self.cards.get(&norm(&v.fr))).map(|c| c.level as u32).sum();
        if sum == 0 { return 0; }
        let avg = sum as f32 / u.vocab.len() as f32;
        if avg >= 5.0 { 3 } else if avg >= 3.0 { 2 } else { 1 }
    }

    /* ── Percobaan & metrik ── */
    pub fn record(&mut self, kind: &str, tag: &str, ok: bool) {
        self.attempts.push(Attempt { t: now_ms(), kind: kind.into(), tag: tag.into(), ok });
        if self.attempts.len() > 4000 { let n = self.attempts.len() - 4000; self.attempts.drain(0..n); }
        let d = self.day_mut(); d.n += 1; if ok { d.ok += 1; }
    }
    pub fn mark_active(&mut self, text: &str) {
        for w in norm(text).split(|c: char| !(c.is_alphabetic() || c == '\'' || c == '-')) {
            if w.chars().count() > 1 { self.active.insert(w.to_string()); }
        }
    }
    pub fn metrics(&self) -> Metrics {
        let mastered = self.cards.values().filter(|c| c.level >= 5).count();
        let mut by_tag: BTreeMap<String, (u32, u32)> = BTreeMap::new();
        let mut by_skill: BTreeMap<&'static str, (u32, u32)> = ["baca", "dengar", "tulis", "bicara"].iter().map(|k| (*k, (0, 0))).collect();
        for a in &self.attempts {
            if !a.tag.is_empty() { let t = by_tag.entry(a.tag.clone()).or_default(); t.0 += a.ok as u32; t.1 += 1; }
            if let Some(sk) = skill_of(&a.kind) { let t = by_skill.get_mut(sk).unwrap(); t.0 += a.ok as u32; t.1 += 1; }
        }
        let this = week_start(today());
        let mut weeks: Vec<(NaiveDate, Day, u32)> = (0..12).rev().map(|i| (this - Duration::weeks(i), Day::default(), 0)).collect();
        for (k, d) in &self.days {
            if let Ok(date) = NaiveDate::parse_from_str(k, "%Y-%m-%d") {
                let wk = week_start(date);
                if let Some(w) = weeks.iter_mut().find(|w| w.0 == wk) {
                    w.1.sec += d.sec; w.1.ok += d.ok; w.1.n += d.n; w.1.speak_sec += d.speak_sec; if d.sec >= 60.0 { w.2 += 1; }
                }
            }
        }
        Metrics { mastered, cards: self.cards.len(), active: self.active.len(), by_tag, by_skill, weeks }
    }
    /// (ketepatan, jumlah soal, tier perkiraan; -1 = Tier 0 sedang berjalan)
    pub fn estimate_skill(&self, skill: &str, m: &Metrics) -> (f32, u32, i32) {
        let (ok, n) = m.by_skill.get(skill).copied().unwrap_or((0, 0));
        let acc = if n > 0 { ok as f32 / n as f32 } else { 0.0 };
        let mut tier = -1;
        for t in &content::c().tiers {
            let us = content::units_of_tier(t.n);
            if !us.is_empty() && us.iter().all(|u| self.unit_done(&u.id)) && self.tier_passed(t.n) { tier = t.n as i32; } else { break; }
        }
        if n > 0 && acc < 0.7 { tier = (tier - 1).max(-1); }
        (acc, n, tier)
    }
}
