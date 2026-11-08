use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let obj_path = out_dir.join("macos_menu.o");
        let lib_path = out_dir.join("libmacos_menu.a");

        let status = Command::new("clang")
            .args(&[
                "-c",
                "src/macos_menu.m",
                "-o",
                obj_path.to_str().unwrap(),
                "-fobjc-arc",
                "-O3",
                "-Wno-deprecated-declarations",
            ])
            .status()
            .expect("Failed to compile src/macos_menu.m with clang");

        if !status.success() {
            panic!("clang failed to compile src/macos_menu.m");
        }

        let ar_status = Command::new("ar")
            .args(&[
                "rcs",
                lib_path.to_str().unwrap(),
                obj_path.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to create static library with ar");

        if !ar_status.success() {
            panic!("ar failed to create libmacos_menu.a");
        }

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=macos_menu");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreImage");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=ImageIO");
        println!("cargo:rustc-link-lib=framework=Vision");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rerun-if-changed=src/macos_menu.m");
    }
}
