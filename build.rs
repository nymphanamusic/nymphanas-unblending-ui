use cmake::Config;
use miette::IntoDiagnostic;
use std::path::PathBuf;

fn main() -> miette::Result<()> {
    let include_path = PathBuf::from("external");

    // This assumes all your C++ bindings are in main.rs
    let mut cxx_cfg = autocxx_build::Builder::new(
        "src/unblending.rs",
        &[
            &include_path.join("unblending/unblending/include"),
            &include_path.join("eigen"),
        ],
    )
    .build()
    .into_diagnostic()?;
    println!("cargo:rerun-if-changed=src/unblending.rs");

    cxx_cfg.compiler("g++");
    let mut c_cfg = cc::Build::default();
    c_cfg.compiler("gcc");
    let dst = Config::new(include_path.join("unblending"))
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
    Ok(())
}
