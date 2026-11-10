include!("../build_icon.rs");

fn main() {
    embed_icon();
    // Installer menanam Passerelle.exe hasil build aplikasi; bangun ulang bila exe itu berubah.
    println!("cargo:rerun-if-changed=../target/release/Passerelle.exe");
}
