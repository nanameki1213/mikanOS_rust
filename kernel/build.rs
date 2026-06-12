fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-arg={}/src/hankaku.o", manifest_dir);
    println!("cargo:rerun-if-changed=src/hankaku.o");
}
