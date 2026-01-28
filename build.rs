fn main() {
    // Temporarily disable assembly build for cross-platform compatibility
    // println!("cargo:rerun-if-changed=src/asm/nex_core.s");
    // 
    // // Compile assembly file
    // cc::Build::new()
    //     .file("src/asm/nex_core.s")
    //     .compile("nex_core");
    // 
    // // Link the static library
    // println!("cargo:rustc-link-lib=static=nex_core");
    // println!("cargo:rustc-link-search=native={}", std::env::var("OUT_DIR").unwrap());
}
