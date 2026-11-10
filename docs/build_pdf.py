"""Membangun docs/Passerelle-Dokumentasi-Teknis.pdf.
Jalankan: python docs/build_pdf.py  (butuh reportlab)."""
import os
from reportlab.lib.pagesizes import A4
from reportlab.lib.units import mm
from reportlab.lib import colors
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (BaseDocTemplate, Frame, PageTemplate, Paragraph, Spacer, Table, TableStyle,
                                Image, PageBreak, KeepTogether, NextPageTemplate)

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "Passerelle-Dokumentasi-Teknis.pdf")
FONTS = os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts")

pdfmetrics.registerFont(TTFont("UI", os.path.join(FONTS, "segoeui.ttf")))
pdfmetrics.registerFont(TTFont("UI-B", os.path.join(FONTS, "segoeuib.ttf")))
pdfmetrics.registerFont(TTFont("UI-I", os.path.join(FONTS, "segoeuii.ttf")))
pdfmetrics.registerFont(TTFont("UI-Black", os.path.join(FONTS, "seguibl.ttf")))
pdfmetrics.registerFont(TTFont("Serif", os.path.join(FONTS, "georgia.ttf")))
pdfmetrics.registerFont(TTFont("Mono", os.path.join(FONTS, "consola.ttf")))
pdfmetrics.registerFontFamily("UI", normal="UI", bold="UI-B", italic="UI-I", boldItalic="UI-B")

BLUE = colors.HexColor("#2B54F5"); BLUE_SOFT = colors.HexColor("#E6ECFF"); INK = colors.HexColor("#1B1E2B")
INK2 = colors.HexColor("#585F73"); LINE = colors.HexColor("#EEE2CF"); PAPER = colors.HexColor("#FFF9F0")
SUN = colors.HexColor("#FFC23A"); SUN_SOFT = colors.HexColor("#FFF0C7"); RED = colors.HexColor("#F23A48")
CODE_BG = colors.HexColor("#F4F1EA")

st = {
    "h1": ParagraphStyle("h1", fontName="UI-Black", fontSize=20, leading=25, textColor=INK, spaceBefore=4, spaceAfter=8, keepWithNext=1),
    "h2": ParagraphStyle("h2", fontName="UI-B", fontSize=13.5, leading=18, textColor=INK, spaceBefore=12, spaceAfter=5, keepWithNext=1),
    "p": ParagraphStyle("p", fontName="UI", fontSize=10, leading=14.6, textColor=INK, spaceAfter=6, alignment=TA_LEFT),
    "small": ParagraphStyle("small", fontName="UI", fontSize=8.6, leading=12, textColor=INK2, spaceAfter=4),
    "li": ParagraphStyle("li", fontName="UI", fontSize=10, leading=14.4, textColor=INK, leftIndent=12, bulletIndent=2, spaceAfter=2),
    "code": ParagraphStyle("code", fontName="Mono", fontSize=8.4, leading=11.2, textColor=INK, backColor=CODE_BG,
                           borderPadding=(6, 7, 6, 7), spaceBefore=4, spaceAfter=10),
    "cell": ParagraphStyle("cell", fontName="UI", fontSize=8.8, leading=12, textColor=INK),
    "cellb": ParagraphStyle("cellb", fontName="UI-B", fontSize=8.6, leading=11.5, textColor=INK2),
    "cap": ParagraphStyle("cap", fontName="UI-I", fontSize=8.4, leading=11, textColor=INK2, spaceBefore=3, spaceAfter=10),
}

def P(t, s="p"): return Paragraph(t, st[s])
def bullets(items): return [Paragraph(i, st["li"], bulletText="•") for i in items]
def code(t): return Paragraph(t.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\n", "<br/>").replace(" ", "&nbsp;"), st["code"])

def table(rows, widths, head=True):
    data = [[Paragraph(str(c), st["cellb" if (head and r == 0) else "cell"]) for c in row] for r, row in enumerate(rows)]
    t = Table(data, colWidths=widths, repeatRows=1 if head else 0)
    style = [("VALIGN", (0, 0), (-1, -1), "TOP"), ("LINEBELOW", (0, 0), (-1, -1), 0.5, LINE),
             ("TOPPADDING", (0, 0), (-1, -1), 4), ("BOTTOMPADDING", (0, 0), (-1, -1), 4), ("LEFTPADDING", (0, 0), (-1, -1), 5)]
    if head: style += [("BACKGROUND", (0, 0), (-1, 0), BLUE_SOFT), ("LINEBELOW", (0, 0), (-1, 0), 1, BLUE)]
    t.setStyle(TableStyle(style))
    return t

def shot(name, caption, w=165 * mm):
    path = os.path.join(HERE, "img", name)
    from reportlab.lib.utils import ImageReader
    iw, ih = ImageReader(path).getSize()
    img = Image(path, width=w, height=w * ih / iw)
    return KeepTogether([img, P(caption, "cap")])

def note(t, bg=SUN_SOFT, bar=SUN):
    tb = Table([[P(t, "p")]], colWidths=[165 * mm])
    tb.setStyle(TableStyle([("BACKGROUND", (0, 0), (-1, -1), bg), ("LINEBEFORE", (0, 0), (0, -1), 3, bar),
                            ("LEFTPADDING", (0, 0), (-1, -1), 9), ("TOPPADDING", (0, 0), (-1, -1), 6), ("BOTTOMPADDING", (0, 0), (-1, -1), 2)]))
    return tb

W, H = A4
def cover(c, doc):
    c.saveState()
    c.setFillColor(PAPER); c.rect(0, 0, W, H, fill=1, stroke=0)
    for i, col in enumerate([BLUE, LINE, RED]): c.setFillColor(col); c.rect(i * W / 3, H - 6, W / 3 + 1, 6, fill=1, stroke=0)
    # logo
    x, y, s = 22 * mm, H - 62 * mm, 26 * mm
    c.setFillColor(BLUE); c.roundRect(x, y, s, s, 6 * mm / 1.0 * 0.23 * 4, fill=1, stroke=0)
    c.setStrokeColor(colors.white); c.setLineWidth(3.2); c.setLineCap(1)
    k = s / 24.0
    c.line(x + 4 * k, y + (24 - 17) * k, x + 20 * k, y + (24 - 17) * k)
    c.arc(x + 6.5 * k, y + (24 - 17) * k - 7 * k, x + 17.5 * k, y + (24 - 17) * k + 7 * k, 0, 180)
    c.line(x + 12 * k, y + 7 * k, x + 12 * k, y + 14 * k)
    c.setFillColor(SUN); c.circle(x + 18.6 * k, y + (24 - 5.4) * k, 2.3 * k, fill=1, stroke=0)
    c.setFillColor(INK); c.setFont("UI-Black", 34); c.drawString(22 * mm, H - 88 * mm, "Passerelle")
    c.setFont("UI", 15); c.setFillColor(INK2); c.drawString(22 * mm, H - 99 * mm, "Dokumentasi Teknis · versi 1.0.0")
    c.setFont("Serif", 13); c.setFillColor(INK); c.drawString(22 * mm, H - 116 * mm, "« Petit à petit, l’oiseau fait son nid. »")
    c.setFont("UI", 10.5); c.setFillColor(INK2)
    lines = ["Aplikasi Windows native untuk belajar bahasa Prancis dari nol,",
             "dibangun dengan Rust dan egui — tanpa WebView, tanpa server, tanpa biaya berjalan."]
    for i, l in enumerate(lines): c.drawString(22 * mm, H - 128 * mm - i * 6 * mm, l)
    c.setFillColor(SUN_SOFT); c.roundRect(22 * mm, 40 * mm, W - 44 * mm, 42 * mm, 3 * mm, fill=1, stroke=0)
    c.setFillColor(INK); c.setFont("UI-B", 10.5); c.drawString(28 * mm, 72 * mm, "Ringkasan teknis")
    c.setFont("UI", 9.5)
    facts = ["Bahasa: Rust 1.99 (toolchain x86_64-pc-windows-gnu, link dengan rust-lld)",
             "Antarmuka: egui / eframe 0.31 (OpenGL lewat glow) · Audio: WASAPI (cpal) · TTS: WinRT",
             "Distribusi: installer Rust (HKCU, tanpa admin) + versi portabel · ZIP 11,5 MB",
             "Data lokal: %APPDATA%\\Passerelle (JSON + WAV)"]
    for i, f in enumerate(facts): c.drawString(28 * mm, 64 * mm - i * 5.6 * mm, f)
    c.setFont("UI", 8.5); c.setFillColor(INK2); c.drawString(22 * mm, 22 * mm, "Disusun 11 November 2026 · Proyek pribadi Darryl Fatha")
    c.restoreState()

def page(c, doc):
    c.saveState()
    for i, col in enumerate([BLUE, LINE, RED]): c.setFillColor(col); c.rect(i * W / 3, H - 3, W / 3 + 1, 3, fill=1, stroke=0)
    c.setFont("UI", 8); c.setFillColor(INK2)
    c.drawString(22 * mm, 12 * mm, "Passerelle · Dokumentasi Teknis")
    c.drawRightString(W - 22 * mm, 12 * mm, f"Halaman {doc.page}")
    c.restoreState()

doc = BaseDocTemplate(OUT, pagesize=A4, leftMargin=22 * mm, rightMargin=22 * mm, topMargin=18 * mm, bottomMargin=20 * mm,
                      title="Passerelle — Dokumentasi Teknis", author="Darryl Fatha", subject="Arsitektur, build, dan distribusi Passerelle 1.0.0")
frame = Frame(doc.leftMargin, doc.bottomMargin, doc.width, doc.height, id="f")
doc.addPageTemplates([PageTemplate("cover", frames=[frame], onPage=cover), PageTemplate("body", frames=[frame], onPage=page)])

s = [NextPageTemplate("body"), PageBreak()]

# 1
s += [P("1. Ringkasan", "h1"),
      P("Passerelle (\u201cjembatan\u201d) membawa penutur bahasa Indonesia dari nol menuju kemampuan berbahasa Prancis lewat jalur bertingkat "
        "mengikuti CEFR, latihan bicara dengan rekaman lokal, kamus Prancis–Indonesia, dan dasbor progres. Semua berjalan di komputer pengguna: "
        "tidak ada akun, server, kunci API, atau pengumpulan data pemakaian."),
      P("Versi 1.0.0 adalah aplikasi <b>Windows native</b> yang ditulis dengan Rust. Antarmuka digambar langsung oleh egui di atas OpenGL, "
        "bukan halaman web di dalam WebView. Konten kurikulum dan kamus ditanam ke dalam berkas .exe sehingga aplikasi bisa dijalankan tanpa internet sejak awal."),
      P("Prinsip yang dipegang", "h2")]
s += bullets(["<b>Satu langkah berikutnya.</b> Beranda selalu punya satu tombol utama \u201cLanjutkan\u201d yang dihitung dari progres.",
              "<b>Bicara sejak hari pertama.</b> Setiap unit ditutup dengan dialog bercabang yang bisa dijawab dengan klik atau suara.",
              "<b>Progres yang jujur.</b> Angka di layar diturunkan dari jawaban nyata (kata dikuasai, menit bicara, ketepatan per jenis kesalahan).",
              "<b>Nol biaya berjalan.</b> Tidak ada layanan berbayar; TTS dan mikrofon memakai komponen Windows.",
              "<b>Ceria tapi tidak \u201cslop\u201d.</b> Palet cerah untuk remaja, tanpa gradasi ungu, efek kaca, maskot, atau emoji sebagai ikon."])

# 2
s += [P("2. Arsitektur", "h1"),
      table([["Lapisan", "Pilihan", "Catatan"],
             ["Bahasa", "Rust 1.99 (edisi 2021)", "Toolchain stable-x86_64-pc-windows-gnu di D:\\rust"],
             ["Jendela & UI", "eframe / egui 0.31, backend glow (OpenGL)", "Immediate-mode; font Segoe UI & Georgia dari sistem"],
             ["Ikon", "egui_extras (SVG via resvg)", "Ikon garis 1,5 px, diwarnai dengan tint"],
             ["Suara keluar", "tts 0.26 (WinRT SpeechSynthesizer)", "Memilih suara fr-FR bila terpasang"],
             ["Mikrofon & putar ulang", "cpal 0.15 (WASAPI)", "Rekaman disimpan WAV 16-bit lewat hound"],
             ["Data", "serde_json", "state.json di %APPDATA%\\Passerelle"],
             ["Dialog berkas", "rfd 0.15", "Ekspor/impor cadangan progres"],
             ["Installer", "eframe + winreg", "Biner Rust terpisah yang menanam Passerelle.exe"]],
            [32 * mm, 58 * mm, 75 * mm]),
      Spacer(1, 6),
      P("Alur satu frame", "h2"),
      P("eframe memanggil <font face='Mono'>App::update</font> setiap frame. Fungsi ini (1) menerapkan tema bila pengaturan berubah, "
        "(2) membaca pintasan global, (3) menggambar satu dari empat mode — orientasi, sesi latihan, dialog, atau layar utama dengan bilah sisi — "
        "lalu (4) modal, kartu kamus kecil, toast, (5) menjalankan antrean aksi (<font face='Mono'>Act</font>) yang dikumpulkan selama menggambar, "
        "dan (6) menyimpan otomatis tiap dua detik bila status berubah. Pola antrean aksi menghindari peminjaman ganda <font face='Mono'>&amp;mut self</font> di dalam closure UI.")]

# 3
s += [P("3. Struktur proyek", "h1"),
      code("""passerelle-rs/
├─ Cargo.toml            workspace: app + installer, profil release (LTO thin, strip)
├─ env.ps1               variabel build: RUSTUP_HOME, CARGO_HOME, dlltool, rust-lld
├─ build_icon.rs         membuat .res berisi ikon (dipakai kedua build.rs)
├─ app/
│  ├─ build.rs           tanam ikon ke Passerelle.exe
│  ├─ assets/content.json   kurikulum, dialog, kamus, indeks bentuk kata
│  └─ src/
│     ├─ main.rs         App, bilah sisi, pintasan, kartu kamus, toast, simpan otomatis
│     ├─ content.rs      struktur konten, normalisasi, lookup & pencarian kamus
│     ├─ store.rs        status, ulasan berjarak, rangkaian hari, metrik, langkah berikutnya
│     ├─ lesson.rs       penyusun soal, pemutar sesi, dialog bercabang
│     ├─ screens.rs      Beranda, Jalur, Bicara, Kamus, Progres, Pengaturan, orientasi, modal
│     ├─ audio.rs        TTS, perekam, pemutar, bunyi umpan balik, riwayat rekaman
│     └─ ui.rs           token warna, font, ikon, tombol bertekanan, widget bersama
├─ installer/src/main.rs pasang, shortcut, registry, lepas pasang, mode --silent
├─ docs/                 dokumen ini, skrip pembuatnya, tangkapan layar
└─ dist/                 Passerelle-1.0.0-win64.zip, BACA-SAYA.txt""")]

# 4
s += [P("4. Konten dan kurikulum", "h1"),
      P("Konten ditulis manual lalu diekspor ke <font face='Mono'>app/assets/content.json</font> (89 KB) dan ditanam dengan "
        "<font face='Mono'>include_str!</font>. Saat pertama diakses, JSON di-parse sekali ke <font face='Mono'>OnceLock&lt;Content&gt;</font>."),
      table([["Tier", "CEFR", "Nama", "Unit tersedia", "Status"],
             ["0", "Pre-A1", "Premiers pas · Langkah pertama", "4 dari 4", "Lengkap"],
             ["1", "A1", "Découverte · Pemula", "3 dari 12", "9 unit tampil terkunci, \u201csedang ditulis\u201d"],
             ["2–6", "A2–C2", "Survie … Maîtrise", "0", "Terkunci sampai tier sebelumnya lulus"]],
            [14 * mm, 16 * mm, 55 * mm, 28 * mm, 52 * mm]),
      Spacer(1, 6),
      P("Setiap unit memuat 12 kosakata, 6 kalimat, 3–4 soal rumpang berlabel jenis kesalahan (gender, konjugasi, preposisi, ejaan, kosakata), "
        "satu catatan tata bahasa dalam bahasa Indonesia, dan satu dialog bercabang. Unit selesai setelah 7 langkah: 5 pelajaran, dialog, dan ulasan unit."),
      P("Penyusunan pelajaran", "h2"),
      table([["Pelajaran", "Fokus", "Jenis soal"],
             ["1", "Kenali kata (6 kata pertama)", "Pasangkan, apa artinya, pilih bahasa Prancis, dengar, ketik, ucapkan"],
             ["2", "Kenali kata (6 kata berikutnya)", "Sama seperti di atas + susun kalimat"],
             ["3", "Bentuk kalimat", "Susun kalimat, isi rumpang, dengar kalimat, ucapkan"],
             ["4", "Tulis sendiri", "Terjemahkan dengan mengetik, isi rumpang, susun"],
             ["5", "Campuran + ucap", "Semua jenis, dua soal ucapkan"]],
            [20 * mm, 50 * mm, 95 * mm]),
      Spacer(1, 6),
      P("Aturan sesi: tanpa nyawa — soal yang salah ditambahkan lagi ke akhir antrean; ketepatan dihitung dari jawaban benar pada percobaan pertama. "
        "Di Tier 0–1 aksen yang terlewat hanya diberi catatan, dan salah ketik satu huruf pada kalimat panjang (&gt; 8 huruf) tetap dianggap benar "
        "dengan catatan. Ujian tier berisi 18 soal campuran; lulus pada 80% dan membuka tier berikutnya. \u201cUjian lompat\u201d memakai ujian yang sama "
        "dan bila lulus menandai semua unit tier itu selesai.")]

s += [shot("soal-salah.png", "Gambar 1. Jawaban belum tepat ditandai ikon silang, teks \u201cBelum tepat\u201d, dan jawaban yang benar — tidak hanya warna."),
      shot("dialog.png", "Gambar 2. Dialog bercabang: balasan yang kurang pas memunculkan petunjuk kuning lalu percakapan berlanjut.")]

# 5
s += [P("5. Ulasan berjarak, rangkaian hari, dan metrik", "h1"),
      P("Ulasan berjarak", "h2"),
      P("Setiap kata dari pelajaran 1–2 dan dari kamus menjadi kartu dengan tingkat 1–6. Jawaban benar menaikkan tingkat satu dan menjadwalkan ulang "
        "sesuai interval <font face='Mono'>[0, 1, 2, 4, 8, 16, 32]</font> hari; jawaban salah menurunkan dua tingkat (minimal 1) dan memunculkan kartu lagi "
        "10 menit kemudian. Tingkat 1–2 = Dikenal, 3–4 = Terlatih, 5–6 = Dikuasai. \u201cUlasan hari ini\u201d mengambil maksimal 15 kartu jatuh tempo "
        "dan bergiliran antara pilih arti, ketik, dan dengar."),
      P("Rangkaian hari dengan hari jeda", "h2"),
      P("Hari dianggap aktif bila ada minimal 60 detik belajar. Penghitungan berjalan mundur dari hari ini (hari ini yang belum aktif tidak memutus rangkaian); "
        "hari tidak aktif boleh ditutup oleh <b>dua hari jeda per minggu</b> (Senin–Minggu). Hari jeda yang jatuh sebelum hari aktif pertama tidak ditampilkan."),
      P("Metrik di layar Progres", "h2"),
      table([["Metrik", "Cara hitung"],
             ["Kosakata dikuasai", "Jumlah kartu tingkat ≥ 5"],
             ["Kosakata aktif", "Kata berbeda yang benar pada soal ketik, susun, ucapkan, atau dialog"],
             ["Menit bicara & kata/menit", "Total durasi rekaman; kata/menit = jumlah kata kalimat ÷ durasi (perkiraan, tanpa pengenal suara)"],
             ["Ketepatan per jenis", "Setiap percobaan disimpan dengan label jenis kesalahan (maks. 4.000 percobaan)"],
             ["Perkiraan tier per keterampilan", "Tier tertinggi yang lulus ujian; turun satu bila ketepatan keterampilan &lt; 70%"],
             ["Tren 12 minggu", "Menit belajar (batang) dan ketepatan mingguan (garis)"]],
            [45 * mm, 120 * mm]),
      Spacer(1, 4),
      P("Tiga jenis kesalahan teratas punya tombol \u201cLatih ini\u201d yang menyusun 8 soal berlabel sama.")]

# 6
s += [P("6. Kamus", "h1"),
      P("130 entri terkurasi dengan arti, IPA, panduan bunyi berejaan Indonesia, kelas kata, gender, register, tier, kekerapan, contoh, ungkapan, "
        "sinonim/antonim, peringatan kata menipu (faux amis), etimologi, dan tabel konjugasi enam kala untuk kata kerja. Indeks 629 bentuk kata "
        "memetakan bentuk turunan ke entri dasar (<i>suis</i> → <i>être</i>, <i>yeux</i> → <i>œil</i>)."),
      P("Pencarian", "h2")]
s += bullets(["Normalisasi: huruf kecil, aksen dibuang, œ → oe, apostrof tipografis → apostrof biasa (<i>ecole</i> menemukan <i>école</i>).",
              "Urutan skor: cocok persis → bentuk turunan → arti Indonesia persis → awalan → kata dalam arti → salah ketik 1 huruf (Levenshtein) → arti mengandung kata.",
              "Elisi dikenali: <i>l’, d’, j’, m’, t’, s’, c’, n’, qu’</i>; kata bertanda hubung dicari dari bagian pertama.",
              "Setiap kata Prancis di layar bisa diklik untuk membuka kartu kamus kecil dengan tombol \u201cSimpan untuk ulasan\u201d. Ctrl+K membuka kamus cepat dari mana pun."])
s += [shot("kamus.png", "Gambar 3. Pencarian dua arah: \u201ckucing\u201d menemukan chat, lengkap dengan peringatan kata menipu.")]

# 7
s += [P("7. Audio dan latihan bicara", "h1")]
s += bullets(["<b>TTS:</b> crate <font face='Mono'>tts</font> memakai WinRT. Saat mulai, aplikasi mencari suara fr-FR; tombol pelan memakai laju antara min dan normal sesuai pengaturan (default 60%). Tanpa suara Prancis, aplikasi menampilkan panduan bunyi tertulis dan petunjuk cara memasang suara.",
              "<b>Perekam:</b> cpal membuka mikrofon bawaan (format f32/i16/u16), mengambil kanal pertama, menghitung RMS per blok untuk meteran level dan rasio \u201cbersuara\u201d (RMS &gt; 0,02). Rasio &lt; 6% memicu peringatan suara terlalu pelan.",
              "<b>Pemutar:</b> sampel diputar ulang dengan interpolasi linear ke laju keluaran perangkat; bunyi benar/salah dibuat langsung sebagai nada segitiga + sinus.",
              "<b>Penilaian:</b> versi ini belum punya pengenal suara lokal. Pengguna membandingkan rekamannya dengan contoh lalu menekan \u201cSaya sudah mengucapkannya dengan benar\u201d. Mikrofon bermasalah → soal berubah menjadi soal ketik.",
              "<b>Riwayat:</b> maksimal 60 rekaman WAV di <font face='Mono'>%APPDATA%\\Passerelle\\rekaman</font>, bisa diputar dan dihapus satu per satu atau sekaligus."])

# 8
s += [P("8. Sistem desain", "h1"),
      P("Semua warna dan ukuran didefinisikan sebagai token di <font face='Mono'>ui.rs</font> (struct <font face='Mono'>Pal</font>, tema terang dan gelap). "
        "Sudut maksimal 8 px, tombol punya \u201calas\u201d 3 px yang turun saat ditekan, animasi 150–400 ms dan bisa dimatikan."),
      table([["Token", "Terang", "Gelap", "Tugas"],
             ["paper", "#FFF9F0", "#12141C", "Latar halaman (krem hangat)"],
             ["blue", "#2B54F5", "#7C95FF", "Tindakan utama, progres, jawaban benar"],
             ["red", "#F23A48", "#FF6B76", "Perhatian saja: belum tepat, sedang merekam"],
             ["sun", "#FFC23A", "#FFC94D", "Rangkaian hari, target, perayaan"],
             ["tier 0–6", "#FFB020 · #2B54F5 · #FF6B4A · #0F9E8A · #E8820C · #D93F78 · #1B2A6B", "sedikit lebih terang", "Identitas tiap tier di Jalur"]],
            [22 * mm, 62 * mm, 22 * mm, 59 * mm]),
      Spacer(1, 6),
      P("Tipografi: Segoe UI (antarmuka), Segoe UI Black (judul), Georgia (semua teks Prancis, agar bahasa yang dipelajari terlihat berbeda). "
        "Tipografi Prancis dijaga: spasi sempit sebelum ! dan ?, tanda kutip « ». Aksesibilitas: benar/salah selalu dengan ikon + teks, ukuran huruf 100–150% "
        "(zoom factor), tema gelap, semua alur bisa dengan papan ketik (Ctrl+1–5, Ctrl+, Ctrl+K, Enter, 1–5, Spasi, Esc)."),
      shot("beranda.png", "Gambar 4. Beranda tema terang: satu tombol utama, ulasan hari ini, kalimat hari ini, target harian, dan rangkaian hari."),
      shot("jalur.png", "Gambar 5. Jalur belajar: simpul unit berwarna tier dengan alas yang bisa ditekan dan cincin progres.")]

# 9
s += [P("9. Penyimpanan dan privasi", "h1"),
      P("Status disimpan sebagai satu berkas JSON. Penulisan aman: tulis ke <font face='Mono'>state.tmp</font> lalu rename. "
        "Simpan otomatis setiap 2 detik hanya bila isinya berubah, dan sekali lagi saat jendela ditutup."),
      code("""%APPDATA%\\Passerelle\\
├─ state.json        { v, profile, settings, progress{units,tiers,dialog_skip_streak},
│                      cards{kata→Card}, attempts[], days{tanggal→Day}, active[], dict_history,
│                      dict_saved, speak }
└─ rekaman\\
   ├─ daftar.json    [{ id, text, kind, sec, file }]
   └─ <id>.wav       16-bit mono"""),
      P("Ekspor/impor membuat atau membaca cadangan .json yang sama (BOM UTF-8 ditoleransi). \u201cHapus semua data saya\u201d menghapus folder ini. "
        "Tidak ada koneksi jaringan di kode aplikasi.")]

# 10
s += [P("10. Build dari sumber", "h1"),
      P("Komputer pengembang tidak memakai Visual Studio Build Tools. Rust dipasang di D:\\rust dengan toolchain GNU minimal, dan dua masalah toolchain "
        "diselesaikan hanya dengan alat bawaan Rust:"),
      table([["Masalah", "Gejala", "Penyelesaian"],
             ["dlltool GNU butuh assembler (as) yang tidak ikut", "Crate windows-link gagal: \u201cdlltool could not create import library\u201d",
              "<font face='Mono'>-C dlltool=D:\\rust\\tools\\llvm-dlltool.exe</font> (salinan llvm-ar resmi Rust; berperilaku sebagai llvm-dlltool)"],
             ["GNU ld bawaan salah membaca import library buatan llvm-dlltool", "Exe crash 0xC0000005 sebelum main; 26 slot IAT tanpa import descriptor",
              "Link langsung dengan <font face='Mono'>-C linker=rust-lld -C linker-flavor=ld.lld</font>"],
             ["Tidak ada windres/rc untuk ikon", "Exe tampil dengan ikon generik",
              "build_icon.rs merender logo (resvg) ke PNG 16–256 px, menulis .res (RT_ICON + RT_GROUP_ICON) dan meneruskannya ke linker"]],
            [45 * mm, 55 * mm, 65 * mm]),
      Spacer(1, 6),
      code("""# PowerShell, dari folder passerelle-rs
. .\\env.ps1
cargo build --release -p passerelle          # 1. aplikasi  -> target\\release\\Passerelle.exe
cargo build --release -p passerelle-setup    # 2. installer -> target\\release\\Passerelle-Setup.exe
python docs\\build_pdf.py                      # dokumen ini"""),
      note("Urutan penting: installer menanam <font face='Mono'>target/release/Passerelle.exe</font> dengan <font face='Mono'>include_bytes!</font>, "
           "jadi aplikasi harus dibangun lebih dulu. Untuk diagnosis awal-proses, set variabel <font face='Mono'>PASSERELLE_TRACE=1</font> "
           "agar penanda ditulis ke trace.log di folder data.")]

# 11
s += [P("11. Installer dan distribusi", "h1")]
s += bullets(["Memasang ke <font face='Mono'>%LOCALAPPDATA%\\Programs\\Passerelle</font> (folder bisa diubah), tanpa hak administrator.",
              "Menghentikan Passerelle yang sedang berjalan, menyalin Passerelle.exe, dan menyalin dirinya sebagai \u201cUninstall Passerelle.exe\u201d.",
              "Membuat shortcut Desktop dan Start Menu lewat WScript.Shell (folder Desktop diambil dari Windows, jadi aman untuk Desktop yang dialihkan ke OneDrive).",
              "Mendaftar di <font face='Mono'>HKCU\\…\\Uninstall\\Passerelle</font> (nama, versi, ikon, ukuran, UninstallString, QuietUninstallString).",
              "Lepas pasang menghapus shortcut, entri registry, dan folder program; progres hanya dihapus bila pengguna memilih.",
              "Argumen: <font face='Mono'>--silent</font> (pasang tanpa jendela), <font face='Mono'>--uninstall</font>, <font face='Mono'>--uninstall --silent</font>."])
s += [table([["Berkas", "Ukuran", "Isi"],
             ["Passerelle.exe", "8,7 MB", "Aplikasi lengkap (konten tertanam)"],
             ["Passerelle-Setup-1.0.0.exe", "15,8 MB", "Installer + salinan aplikasi"],
             ["Passerelle-1.0.0-win64.zip", "11,5 MB", "Installer, Portable\\Passerelle.exe, BACA-SAYA.txt"]],
            [55 * mm, 22 * mm, 88 * mm]),
      Spacer(1, 6),
      shot("installer.png", "Gambar 6. Installer Rust native dengan pilihan folder dan sakelar shortcut.", w=110 * mm),
      note("Installer belum ditandatangani sertifikat code signing, sehingga Windows SmartScreen menampilkan peringatan saat pertama dijalankan.")]

# 12
s += [P("12. Pengujian", "h1"),
      P("Pengujian dilakukan pada build release di Windows 11 dengan folder data uji terpisah, menggerakkan jendela lewat klik dan ketikan otomatis lalu memeriksa tangkapan layar."),
      table([["Bagian", "Hasil"],
             ["Orientasi 5 langkah, tes mikrofon", "Berjalan; meteran level dan peringatan suara pelan muncul"],
             ["Soal pasangan & pilihan ganda (mouse dan tombol 1–5, Enter)", "Benar/salah, antrean ulang, salah-pasang tercatat"],
             ["Dialog bercabang sampai selesai", "Petunjuk cabang keliru, ringkasan, progres unit tersimpan"],
             ["Kamus: bentuk turunan, Indonesia→Prancis, konjugasi", "suis → être, kucing → chat"],
             ["Kartu kamus kecil dari kata yang diklik", "Muncul; kata di luar kamus diberi pesan jujur"],
             ["Jalur, Bicara, Progres, Pengaturan, tema terang/gelap", "Tampil benar setelah perbaikan tata letak"],
             ["Ikon exe, installer UI, shortcut Desktop/Start Menu, registry", "Terverifikasi; aplikasi terbuka dari shortcut Desktop"]],
            [85 * mm, 80 * mm]),
      Spacer(1, 6),
      P("Belum diuji: satu pelajaran penuh sampai layar ringkasan, soal ketik dan susun kalimat dengan input sungguhan, ujian tier, ekspor/impor, "
        "TTS dengan suara Prancis terpasang, dan alur lepas pasang.", "p")]

# 13
s += [P("13. Batasan dan langkah berikutnya", "h1")]
s += bullets(["Konten baru 7 dari 108 unit yang direncanakan; penulisan dan pemeriksaan oleh penutur asli adalah pekerjaan terbesar berikutnya.",
              "Belum ada pengenal suara lokal; rencana: whisper.cpp model base/small atau crate whisper-rs untuk mencocokkan transkrip dengan jawaban yang diterima.",
              "Kamus masih 130 entri terkurasi; rencana: impor Wiktextract/kaikki.org dengan label lapis sumber (terkurasi, Wiktionary, terjemahan otomatis).",
              "Audio contoh bergantung pada suara Windows; rencana: membundel audio yang dibuat sekali dengan Kokoro per tier.",
              "Penandatanganan kode untuk menghilangkan peringatan SmartScreen.",
              "Pengingat harian (notifikasi Windows) dan tes penempatan yang lebih pendek daripada ujian tier."])

doc.build(s)
print("OK", OUT)
