use cmake::Config;

fn main() {
    let mut cxx_cfg = cc::Build::default();
    cxx_cfg.compiler("g++");
    let mut c_cfg = cc::Build::default();
    c_cfg.compiler("gcc");
    let dst = Config::new("include/unblending")
        .no_build_target(true)
        .no_default_flags(true)
        .define("UNBLENDING_BUILD_CLI_APP", "off")
        .define("UNBLENDING_BUILD_GUI_APP", "off")
        .init_cxx_cfg(cxx_cfg)
        .init_c_cfg(c_cfg)
        .generator("Ninja")
        // .out_dir(Path::new("."))
        .build();
    println!(
        "cargo:rustc-link-search=native={}",
        (dst.join("build/unblending")).display()
    );
    println!("cargo:rustc-link-lib=static:+verbatim=libunblending.a");

    // println!("cargo:rerun-if-changed=src/blobstore.cc");
    // println!("cargo:rerun-if-changed=include/blobstore.h");
}
