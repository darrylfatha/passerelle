//! Suara: pengucap teks Windows (WinRT), perekam mikrofon & pemutar (WASAPI lewat cpal),
//! bunyi umpan balik, dan riwayat rekaman WAV di komputer ini saja.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct Take { pub samples: Arc<Vec<f32>>, pub rate: u32, pub sec: f32, pub voiced: f32 }
impl Clone for Take { fn clone(&self) -> Self { Self { samples: self.samples.clone(), rate: self.rate, sec: self.sec, voiced: self.voiced } } }

pub struct Recorder {
    _stream: cpal::Stream,
    buf: Arc<Mutex<Vec<f32>>>,
    pub level: Arc<AtomicU32>,
    voiced: Arc<AtomicU32>,
    chunks: Arc<AtomicU32>,
    rate: u32,
    start: Instant,
}

impl Recorder {
    pub fn start() -> Result<Self, String> {
        let host = cpal::default_host();
        let dev = host.default_input_device().ok_or("Mikrofon tidak ditemukan.")?;
        let sup = dev.default_input_config().map_err(|e| e.to_string())?;
        let fmt = sup.sample_format();
        let cfg: cpal::StreamConfig = sup.into();
        let ch = cfg.channels as usize;
        let rate = cfg.sample_rate.0;
        let buf = Arc::new(Mutex::new(Vec::<f32>::with_capacity(rate as usize * 10)));
        let level = Arc::new(AtomicU32::new(0));
        let voiced = Arc::new(AtomicU32::new(0));
        let chunks = Arc::new(AtomicU32::new(0));
        let (b, l, v, c) = (buf.clone(), level.clone(), voiced.clone(), chunks.clone());
        let push = move |mono: &mut dyn Iterator<Item = f32>| {
            let mut g = b.lock().unwrap();
            let mut sum = 0.0f32; let mut n = 0;
            for s in mono { sum += s * s; n += 1; g.push(s); }
            let rms = if n > 0 { (sum / n as f32).sqrt() } else { 0.0 };
            l.store(rms.to_bits(), Ordering::Relaxed);
            c.fetch_add(1, Ordering::Relaxed);
            if rms > 0.02 { v.fetch_add(1, Ordering::Relaxed); }
        };
        let err = |e| eprintln!("mikrofon: {e}");
        let stream = match fmt {
            cpal::SampleFormat::F32 => { let mut p = push; dev.build_input_stream(&cfg, move |d: &[f32], _: &_| p(&mut d.chunks(ch).map(|f| f[0])), err, None) }
            cpal::SampleFormat::I16 => { let mut p = push; dev.build_input_stream(&cfg, move |d: &[i16], _: &_| p(&mut d.chunks(ch).map(|f| f[0] as f32 / 32768.0)), err, None) }
            cpal::SampleFormat::U16 => { let mut p = push; dev.build_input_stream(&cfg, move |d: &[u16], _: &_| p(&mut d.chunks(ch).map(|f| (f[0] as f32 - 32768.0) / 32768.0)), err, None) }
            _ => return Err("Format mikrofon tidak didukung.".into()),
        }.map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        Ok(Self { _stream: stream, buf, level, voiced, chunks, rate, start: Instant::now() })
    }
    pub fn level(&self) -> f32 { f32::from_bits(self.level.load(Ordering::Relaxed)) }
    pub fn elapsed(&self) -> f32 { self.start.elapsed().as_secs_f32() }
    pub fn stop(self) -> Take {
        let samples = std::mem::take(&mut *self.buf.lock().unwrap());
        let total = self.chunks.load(Ordering::Relaxed).max(1);
        let voiced = self.voiced.load(Ordering::Relaxed) as f32 / total as f32;
        let sec = samples.len() as f32 / self.rate as f32;
        Take { samples: Arc::new(samples), rate: self.rate, sec, voiced }
    }
}

pub struct Audio {
    tts: Option<tts::Tts>,
    voice_name: String,
    out: Option<cpal::Stream>,
    pub sound: bool,
    pub slow_rate: f32,
}

impl Audio {
    pub fn new() -> Self {
        let mut voice_name = String::new();
        let tts = tts::Tts::default().ok().and_then(|mut t| {
            let voices = t.voices().ok()?;
            let pick = voices.iter().find(|v| { let l = v.language().to_string(); l.starts_with("fr-FR") })
                .or_else(|| voices.iter().find(|v| v.language().to_string().starts_with("fr")))?.clone();
            voice_name = pick.name();
            t.set_voice(&pick).ok()?;
            Some(t)
        });
        Self { tts, voice_name, out: None, sound: true, slow_rate: 0.6 }
    }
    pub fn has_voice(&self) -> bool { self.tts.is_some() }
    pub fn voice_name(&self) -> &str { &self.voice_name }

    /// Mengembalikan false jika suara Prancis belum terpasang.
    pub fn say(&mut self, text: &str, slow: bool) -> bool {
        let rate_k = self.slow_rate;
        let Some(t) = self.tts.as_mut() else { return false };
        let normal = t.normal_rate();
        let min = t.min_rate();
        let rate = if slow { min + (normal - min) * rate_k } else { normal };
        let _ = t.set_rate(rate);
        let _ = t.speak(text.replace("___", "…"), true);
        true
    }

    pub fn play(&mut self, samples: Arc<Vec<f32>>, rate: u32) {
        self.out = None;
        let host = cpal::default_host();
        let Some(dev) = host.default_output_device() else { return };
        let Ok(sup) = dev.default_output_config() else { return };
        let fmt = sup.sample_format();
        let cfg: cpal::StreamConfig = sup.into();
        let ch = cfg.channels as usize;
        let step = rate as f64 / cfg.sample_rate.0 as f64;
        let pos = Arc::new(AtomicUsize::new(0));
        let next = move || -> f32 {
            let i = pos.fetch_add(1, Ordering::Relaxed);
            let src = i as f64 * step;
            let j = src as usize;
            if j + 1 >= samples.len() { return 0.0; }
            let f = (src - j as f64) as f32;
            samples[j] * (1.0 - f) + samples[j + 1] * f
        };
        let err = |e| eprintln!("pengeras suara: {e}");
        let stream = match fmt {
            cpal::SampleFormat::F32 => { let mut n = next; dev.build_output_stream(&cfg, move |o: &mut [f32], _: &_| for fr in o.chunks_mut(ch) { let s = n(); fr.iter_mut().for_each(|x| *x = s); }, err, None) }
            cpal::SampleFormat::I16 => { let mut n = next; dev.build_output_stream(&cfg, move |o: &mut [i16], _: &_| for fr in o.chunks_mut(ch) { let s = (n() * 32767.0) as i16; fr.iter_mut().for_each(|x| *x = s); }, err, None) }
            _ => return,
        };
        if let Ok(s) = stream { if s.play().is_ok() { self.out = Some(s); } }
    }
    pub fn play_take(&mut self, t: &Take) { self.play(t.samples.clone(), t.rate); }

    fn tones(&mut self, freqs: &[f32], dur: f32, gain: f32) {
        if !self.sound { return; }
        let rate = 48_000u32;
        let mut v = Vec::new();
        for (k, f) in freqs.iter().enumerate() {
            let len = (dur * 1.8 * rate as f32) as usize;
            let start = (k as f32 * dur * rate as f32) as usize;
            if v.len() < start + len { v.resize(start + len, 0.0); }
            for i in 0..len {
                let t = i as f32 / rate as f32;
                let env = (t / 0.01).min(1.0) * (-t * 5.0 / (dur * 1.8)).exp();
                let w = (2.0 * std::f32::consts::PI * f * t).sin();
                let tri = (2.0 / std::f32::consts::PI) * w.asin();
                v[start + i] += gain * env * (0.6 * tri + 0.4 * w);
            }
        }
        v.resize(v.len() + 2000, 0.0);
        self.play(Arc::new(v), rate);
    }
    pub fn ok(&mut self) { self.tones(&[660.0, 990.0], 0.08, 0.16); }
    pub fn bad(&mut self) { self.tones(&[300.0, 240.0], 0.11, 0.12); }
    pub fn tap(&mut self) { self.tones(&[520.0], 0.03, 0.06); }
    pub fn done(&mut self) { self.tones(&[523.0, 659.0, 784.0, 1047.0], 0.09, 0.15); }
}

/* ── Riwayat rekaman ── */
#[derive(Serialize, Deserialize, Clone)]
pub struct RecMeta { pub id: i64, pub text: String, pub kind: String, pub sec: f32, pub file: String }

fn rec_dir() -> PathBuf { let d = crate::store::data_dir().join("rekaman"); let _ = std::fs::create_dir_all(&d); d }
fn rec_index() -> PathBuf { rec_dir().join("daftar.json") }
pub fn recs_all() -> Vec<RecMeta> { std::fs::read_to_string(rec_index()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default() }
fn recs_write(v: &[RecMeta]) { let _ = std::fs::write(rec_index(), serde_json::to_string(v).unwrap_or_default()); }

pub fn rec_save(text: &str, kind: &str, t: &Take) {
    let id = crate::store::now_ms();
    let file = format!("{id}.wav");
    let spec = hound::WavSpec { channels: 1, sample_rate: t.rate, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
    if let Ok(mut w) = hound::WavWriter::create(rec_dir().join(&file), spec) {
        for s in t.samples.iter() { let _ = w.write_sample((s.clamp(-1.0, 1.0) * 32767.0) as i16); }
        let _ = w.finalize();
    }
    let mut all = recs_all();
    all.push(RecMeta { id, text: text.into(), kind: kind.into(), sec: t.sec, file });
    while all.len() > 60 { let r = all.remove(0); let _ = std::fs::remove_file(rec_dir().join(r.file)); }
    recs_write(&all);
}
pub fn rec_load(m: &RecMeta) -> Option<Take> {
    let mut r = hound::WavReader::open(rec_dir().join(&m.file)).ok()?;
    let rate = r.spec().sample_rate;
    let s: Vec<f32> = r.samples::<i16>().filter_map(|x| x.ok()).map(|x| x as f32 / 32768.0).collect();
    Some(Take { sec: s.len() as f32 / rate as f32, samples: Arc::new(s), rate, voiced: 1.0 })
}
pub fn rec_delete(id: i64) {
    let mut all = recs_all();
    if let Some(i) = all.iter().position(|r| r.id == id) { let r = all.remove(i); let _ = std::fs::remove_file(rec_dir().join(r.file)); }
    recs_write(&all);
}
pub fn rec_clear() { let _ = std::fs::remove_dir_all(rec_dir()); }
