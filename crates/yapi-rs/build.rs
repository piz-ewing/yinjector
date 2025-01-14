use std::{env, path::PathBuf};
extern crate bindgen;
extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=ffi/yapi/yapi.hpp");
    println!("cargo:rerun-if-changed=ffi/yapi.cpp");

    println!("cargo:rustc-link-lib=static=yapi");

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let target_arch_define = if target_arch == "x86" {
        "_WIN32".to_owned()
    } else if target_arch == "x86_64" {
        "_WIN64".to_owned()
    } else {
        panic!("Unknown target architecture: {}", target_arch)
    };

    // _C_API
    let out_path = PathBuf::from(
        env::var_os("OUT_DIR").expect("the environment variable OUT_DIR is undefined"),
    );

    bindgen::Builder::default()
        .header("src/wrapper.h")
        // .layout_tests(false)
        .ctypes_prefix("cty")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .clang_arg("-Iffi")
        .clang_arg("-Iffi/yapi")
        .clang_arg("-D__UNICODE=1")
        .clang_arg("-D_UNICODE=1")
        .clang_arg(std::format!("-D_{}=1", &target_arch_define))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(true)
        .impl_debug(true)
        .generate()
        .unwrap()
        .write_to_file(out_path.join("yapi.rs"))
        .unwrap();

    cc::Build::new()
        .define("_UNICODE", Some("1"))
        .define("UNICODE", Some("1"))
        .define(&target_arch_define, Some("1"))
        .file("ffi/wrapper.cpp")
        .include("ffi/yapi")
        .compile("yapi");
}
