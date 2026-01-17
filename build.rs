use cmake::Config;
use miette::IntoDiagnostic;
use std::path::PathBuf;

fn main() -> miette::Result<()> {
    let include_path = PathBuf::from("external");
    let unblending_path = include_path.join("unblending/unblending/include");
    let eigen_path = include_path.join("eigen");
    let source_path = PathBuf::from("src");

    // This assumes all your C++ bindings are in main.rs
    let mut cxx_cfg = autocxx_build::Builder::new(
        "src/unblending.rs",
        &[&unblending_path, &eigen_path, &source_path],
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

    cc::Build::default()
        .cpp(true)
        .compiler("g++")
        .include(&source_path)
        .include(&unblending_path)
        .include(&eigen_path)
        .file("src/unblending_helpers.cpp")
        .compile("unblending_helpers");
    println!("cargo:rerun-if-changed=src/unblending_helpers.hpp");
    println!("cargo:rerun-if-changed=src/unblending_helpers.cpp");
    println!("cargo:rustc-link-lib=static:+verbatim=libunblending_helpers.a");

    Ok(())
}
