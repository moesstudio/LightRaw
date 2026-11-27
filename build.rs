use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let has_desktop = env::var_os("CARGO_FEATURE_DESKTOP").is_some();
    if target_os == "macos" {
        if has_desktop {
            let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
            let obj_path = out_dir.join("macos_menu.o");
            let lib_path = out_dir.join("libmacos_menu.a");
            let clang_arch = match target_arch.as_str() {
                "x86_64" => "x86_64",
                _ => "arm64",
            };
            let min_macos = env::var("MACOSX_DEPLOYMENT_TARGET")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "12.0".to_string());
            let min_flag = format!("-mmacosx-version-min={}", min_macos);

            let status = Command::new("clang")
                .args(&[
                    "-c",
                    "src/macos_menu.m",
                    "-o",
                    obj_path.to_str().unwrap(),
                    "-arch",
                    clang_arch,
                    &min_flag,
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

            // Link Apple's compiler runtime (`libclang_rt.osx.a`) so `@available(macOS ...)`
            // (`___isPlatformVersionAtLeast`) links cleanly when Rust invokes `cc -nodefaultlibs`.
            if let Ok(rt_out) = Command::new("clang").arg("--print-resource-dir").output() {
                if rt_out.status.success() {
                    let res_dir = String::from_utf8_lossy(&rt_out.stdout).trim().to_string();
                    let darwin_dir = PathBuf::from(&res_dir).join("lib").join("darwin");
                    if darwin_dir.join("libclang_rt.osx.a").is_file() {
                        println!("cargo:rustc-link-search=native={}", darwin_dir.display());
                        println!("cargo:rustc-link-lib=static=clang_rt.osx");
                    }
                }
            }
        }
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=CoreImage");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=ImageIO");
        println!("cargo:rustc-link-lib=framework=Vision");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rerun-if-changed=src/macos_menu.m");
        println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    }
}
