use miette::IntoDiagnostic;
use std::path::PathBuf;

fn main() -> miette::Result<()> {
    let include_path = PathBuf::from("external");
    let unblending_path = include_path.join("unblending/unblending/include");
    let eigen_path = include_path.join("eigen");
    let source_path = PathBuf::from("src");

    // 2. Build unblending with CMake (no init_cxx_cfg here, or only for its own files)
    let mut cxx_cfg = cc::Build::default();
    cxx_cfg.compiler("g++");
    let mut c_cfg = cc::Build::default();
    c_cfg.compiler("gcc");
    let dst = cmake::Config::new(include_path.join("unblending"))
        .no_build_target(true)
        .no_default_flags(true)
        .define("CMAKE_POLICY_VERSION_MINIMUM", "3.1")
        .define("UNBLENDING_BUILD_CLI_APP", "off")
        .define("UNBLENDING_BUILD_GUI_APP", "off")
        .init_c_cfg(c_cfg)
        .init_cxx_cfg(cxx_cfg)
        .generator("Ninja")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=unblending");

    // 1. Build autocxx bridge into its own static lib
    let mut bridge = autocxx_build::Builder::new(
        "src/unblending_ffi.rs",
        &[&unblending_path, &eigen_path, &source_path],
    )
    .build()
    .into_diagnostic()?;

    bridge
        .cpp(true)
        .compiler("g++")
        .flag_if_supported("-std=c++14")
        .file("src/unblending_helpers.cpp")
        .compile("unblending_autocxx");

    println!("cargo:rustc-link-lib=unblending_autocxx");

    println!("cargo:rerun-if-changed=src/unblending.rs");
    println!("cargo:rerun-if-changed=src/unblending_ffi.rs");
    println!("cargo:rerun-if-changed=src/unblending_helpers.hpp");
    println!("cargo:rerun-if-changed=src/unblending_helpers.cpp");

    Ok(())
}
