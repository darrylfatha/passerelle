// Dipakai oleh build.rs aplikasi dan installer (lewat include!).
// Menggambar logo Passerelle lalu menulis berkas resource Windows (.res) berisi ikon,
// dan meminta linker (rust-lld) menyertakannya — tanpa butuh windres/rc.

const LOGO: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="256" height="256"><rect width="24" height="24" rx="5.5" fill="#2B54F5"/><path d="M4 17h16M6.5 17c0-4.2 2.5-7 5.5-7s5.5 2.8 5.5 7M12 17v-7" fill="none" stroke="#fff" stroke-width="1.9" stroke-linecap="round"/><circle cx="18.6" cy="5.4" r="2.3" fill="#FFC23A"/></svg>"##;

fn render_png(size: u32) -> Vec<u8> {
    let tree = resvg::usvg::Tree::from_str(LOGO, &resvg::usvg::Options::default()).unwrap();
    let mut pm = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
    let s = size as f32 / tree.size().width();
    resvg::render(&tree, resvg::tiny_skia::Transform::from_scale(s, s), &mut pm.as_mut());
    let mut rgba = pm.data().to_vec();
    for px in rgba.chunks_mut(4) {
        let a = px[3] as f32 / 255.0;
        if a > 0.0 { for c in &mut px[..3] { *c = ((*c as f32 / a).round().min(255.0)) as u8; } }
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, size, size);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&rgba).unwrap();
    }
    out
}

fn res_entry(out: &mut Vec<u8>, ty: u16, name: u16, flags: u16, data: &[u8]) {
    let w32 = |o: &mut Vec<u8>, v: u32| o.extend_from_slice(&v.to_le_bytes());
    let w16 = |o: &mut Vec<u8>, v: u16| o.extend_from_slice(&v.to_le_bytes());
    w32(out, data.len() as u32); // DataSize
    w32(out, 32);                // HeaderSize
    w16(out, 0xFFFF); w16(out, ty);
    w16(out, 0xFFFF); w16(out, name);
    w32(out, 0);                 // DataVersion
    w16(out, flags);             // MemoryFlags
    w16(out, 0x0409);            // LanguageId
    w32(out, 0); w32(out, 0);    // Version, Characteristics
    out.extend_from_slice(data);
    while out.len() % 4 != 0 { out.push(0); }
}

pub fn embed_icon() {
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let pngs: Vec<Vec<u8>> = sizes.iter().map(|s| render_png(*s)).collect();
    let mut res = Vec::new();
    res_entry(&mut res, 0, 0, 0, &[]); // entri kosong pembuka berkas .res
    for (i, p) in pngs.iter().enumerate() { res_entry(&mut res, 3, (i + 1) as u16, 0x1010, p); } // RT_ICON
    let mut grp = Vec::new();
    grp.extend_from_slice(&0u16.to_le_bytes()); grp.extend_from_slice(&1u16.to_le_bytes()); grp.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    for (i, s) in sizes.iter().enumerate() {
        let d = if *s >= 256 { 0u8 } else { *s as u8 };
        grp.extend_from_slice(&[d, d, 0, 0]);
        grp.extend_from_slice(&1u16.to_le_bytes()); grp.extend_from_slice(&32u16.to_le_bytes());
        grp.extend_from_slice(&(pngs[i].len() as u32).to_le_bytes());
        grp.extend_from_slice(&((i + 1) as u16).to_le_bytes());
    }
    res_entry(&mut res, 14, 1, 0x1030, &grp); // RT_GROUP_ICON
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("passerelle-icon.res");
    std::fs::write(&out, res).unwrap();
    println!("cargo:rustc-link-arg-bins={}", out.display());
    println!("cargo:rerun-if-changed=../build_icon.rs");
}
