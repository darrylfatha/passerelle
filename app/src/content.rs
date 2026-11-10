//! Konten kurikulum dan kamus, ditanam ke dalam .exe (assets/content.json).
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Deserialize, Clone)]
pub struct Tier { pub n: u8, pub cefr: String, pub name: String, pub label: String, pub units: u32, pub goal: String }
#[derive(Deserialize, Clone, Default)]
pub struct Item { pub fr: String, pub tr: String, #[serde(default)] pub pron: String }
#[derive(Deserialize, Clone)]
pub struct Fill { pub fr: String, pub answer: String, pub options: Vec<String>, pub tag: String, pub why: String }
#[derive(Deserialize, Clone)]
pub struct Block { pub kind: String, pub text: String, pub rows: Vec<Vec<String>> }
#[derive(Deserialize, Clone)]
pub struct Note { pub title: String, pub blocks: Vec<Block> }
#[derive(Deserialize, Clone)]
pub struct Reply { pub fr: String, pub tr: String, pub next: String, pub hint: String }
#[derive(Deserialize, Clone)]
pub struct DNode { pub say: String, pub tr: String, pub end: bool, pub replies: Vec<Reply> }
#[derive(Deserialize, Clone)]
pub struct Dialog { pub title: String, pub partner: String, pub role: String, pub start: String, pub nodes: HashMap<String, DNode> }
#[derive(Deserialize, Clone)]
pub struct Unit {
    pub id: String, pub tier: u8, pub title: String, pub theme: String,
    pub vocab: Vec<Item>, pub sentences: Vec<Item>, pub fills: Vec<Fill>, pub note: Note, pub dialog: Dialog,
}
#[derive(Deserialize, Clone)]
pub struct Ex { pub fr: String, pub tr: String }
#[derive(Deserialize, Clone)]
pub struct Conj { pub tense: String, pub forms: Vec<String> }
#[derive(Deserialize, Clone)]
pub struct Entry {
    pub w: String, pub key: String, pub pos: String, pub g: String, pub tr: Vec<String>, pub pron: String, pub ipa: String,
    pub def_fr: String, pub reg: String, pub freq: u8, pub tier: u8, pub fem: String, pub pl: String,
    pub ex: Vec<Ex>, pub idi: Vec<Ex>, pub syn: Vec<String>, pub ant: Vec<String>, pub rel: Vec<String>,
    pub faux: String, pub etym: String, pub src: String, pub conj: Vec<Conj>,
}
#[derive(Deserialize, Clone)]
pub struct Pair { pub a: String, pub b: String, pub ta: String, pub tb: String, pub tip: String }
#[derive(Deserialize, Clone)]
pub struct Daily { pub fr: String, pub tr: String, pub note: String }

#[derive(Deserialize)]
pub struct Content {
    pub tiers: Vec<Tier>, pub upcoming: Vec<String>, pub units: Vec<Unit>, pub dict: Vec<Entry>,
    pub forms: HashMap<String, usize>, pub pairs: Vec<Pair>, pub daily: Vec<Daily>,
}

static CONTENT: OnceLock<Content> = OnceLock::new();
pub fn c() -> &'static Content {
    CONTENT.get_or_init(|| serde_json::from_str(include_str!("../assets/content.json")).expect("content.json rusak"))
}

pub fn unit(id: &str) -> Option<&'static Unit> { c().units.iter().find(|u| u.id == id) }
pub fn units_of_tier(t: u8) -> Vec<&'static Unit> { c().units.iter().filter(|u| u.tier == t).collect() }
pub fn all_vocab() -> Vec<Item> { c().units.iter().flat_map(|u| u.vocab.iter().cloned()).collect() }

/// Normalisasi: huruf kecil, tanpa aksen, œ → oe. Sama dengan kunci di content.json.
pub fn norm(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for ch in s.trim().chars().flat_map(|c| c.to_lowercase()) {
        match ch {
            'à' | 'â' | 'ä' | 'á' => o.push('a'),
            'é' | 'è' | 'ê' | 'ë' => o.push('e'),
            'î' | 'ï' | 'í' => o.push('i'),
            'ô' | 'ö' | 'ó' => o.push('o'),
            'ù' | 'û' | 'ü' | 'ú' => o.push('u'),
            'ÿ' => o.push('y'),
            'ç' => o.push('c'),
            'œ' => o.push_str("oe"),
            'æ' => o.push_str("ae"),
            '’' => o.push('\''),
            c => o.push(c),
        }
    }
    o
}

pub fn lookup(raw: &str) -> Option<&'static Entry> {
    let k: String = norm(raw).chars().filter(|c| !"«»\"!?.,;:()".contains(*c)).collect();
    if k.is_empty() { return None; }
    let f = &c().forms;
    if let Some(&i) = f.get(&k) { return c().dict.get(i); }
    for p in ["l'", "d'", "j'", "m'", "t'", "s'", "c'", "n'", "qu'"] {
        if let Some(rest) = k.strip_prefix(p) { if let Some(&i) = f.get(rest) { return c().dict.get(i); } }
    }
    if let Some((a, _)) = k.split_once('-') { if let Some(&i) = f.get(a) { return c().dict.get(i); } }
    None
}

pub fn lev(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + if a[i - 1] == b[j - 1] { 0 } else { 1 });
        }
        prev = cur;
    }
    prev[b.len()]
}

pub struct Hit { pub e: &'static Entry, pub why: &'static str, score: f32 }

/// Pencarian dua arah, toleran aksen dan salah ketik ringan.
pub fn search(q: &str) -> Vec<Hit> {
    let k = norm(q);
    if k.is_empty() { return vec![]; }
    let mut out: Vec<Hit> = Vec::new();
    let mut add = |e: &'static Entry, score: f32, why: &'static str| {
        if let Some(h) = out.iter_mut().find(|h| std::ptr::eq(h.e, e)) {
            if score < h.score { h.score = score; h.why = why; }
        } else { out.push(Hit { e, why, score }); }
    };
    if let Some(e) = lookup(q) { if e.key == k { add(e, 0.0, "") } else { add(e, 1.0, "bentuk dari") } }
    for e in &c().dict {
        if e.key == k { add(e, 0.0, ""); }
        else if e.key.starts_with(&k) { add(e, 2.0, ""); }
        else if k.chars().count() >= 4 && lev(&e.key, &k) == 1 { add(e, 3.0, "mungkin maksudmu"); }
        for t in &e.tr {
            let tk = norm(t);
            if tk == k { add(e, 1.0, "arti"); }
            else if tk.split(|c: char| " ;(),".contains(c)).any(|w| w == k) { add(e, 2.5, "arti"); }
            else if k.chars().count() >= 3 && tk.contains(&k) { add(e, 4.0, "arti"); }
        }
    }
    out.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap().then(b.e.freq.cmp(&a.e.freq)));
    out.truncate(30);
    out
}
