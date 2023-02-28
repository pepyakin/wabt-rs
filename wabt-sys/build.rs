extern crate cc;
extern crate cmake;
#[cfg(windows)]
extern crate glob;

use std::env;
use std::path::Path;
use std::process;

fn main() {
    println!("cargo:rerun-if-env-changed=WABT_CXXSTDLIB");
    println!("cargo:rerun-if-env-changed=CXXSTDLIB");

    let cmake_lists = Path::new("wabt/CMakeLists.txt");
    if !cmake_lists.exists() {
        eprintln!(
            "{} doesn't exist. Perhaps, you need to update git submodules?\n\nTry\n\n\t\
git submodule update --init --recursive",
            cmake_lists.display(),
        );
        process::exit(1);
    }

    let mut cfg = cmake::Config::new("wabt");
    // Turn off building tests and tools. This not only speeds up the build but
    // also prevents building executables that for some reason are put
    // into the `wabt` directory which now is deemed as an error.
    //
    // Since that is all targets available, also don't specify target
    // (by default it is set to `--target install`).
    // Otherwise, there will be an error message "No rule to make target `install'".
    cfg.define("BUILD_TESTS", "OFF")
        .define("BUILD_TOOLS", "OFF")
        .no_build_target(true);

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Can't get the target OS!");
    if target_os == "android" {
        let android_ndk_home = env::var("ANDROID_NDK_HOME").expect("Can't get ANDROID_NDK_HOME!");
        let toolchain = format!("{}/build/cmake/android.toolchain.cmake", android_ndk_home);
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH")
            .expect("Can't get the target architecture of Android!");
        let target_abi = match &*target_arch {
            "aarch64" => "arm64-v8a",
            "arm" => "armeabi-v7a",
            _ => &*target_arch,
        };
        cfg.define("CMAKE_TOOLCHAIN_FILE", toolchain)
            .define("ANDROID_ABI", target_abi);
    };

    // Generally, workaround for https://github.com/rust-lang/cc-rs/pull/506
    // CMake links dynamic debug or release C runtime by default
    // when `cc` crate links dynamic or static release one.
    if target_os == "windows" {
        let is_static_crt = env::var("CARGO_CFG_TARGET_FEATURE")
            .unwrap_or_default()
            .contains("crt-static");
        let msvc_crt = if is_static_crt {
            "MultiThreaded"
        } else {
            "MultiThreadedDLL"
        };
        cfg.define("CMAKE_POLICY_DEFAULT_CMP0091", "NEW")
            .define("CMAKE_MSVC_RUNTIME_LIBRARY", msvc_crt);
    }

    let dst = cfg.build();

    let mut out_build_dir = dst;
    out_build_dir.push("build");

    println!("cargo:rustc-link-search=native={}", out_build_dir.display());

    // help cargo find wabt.lib when targeting windows
    #[cfg(windows)]
    {
        let pattern = format!("{}/*/wabt.lib", out_build_dir.display());
        for entry in glob::glob(&pattern).unwrap() {
            if let Ok(path) = entry {
                let out_lib_dir = path.parent().unwrap().to_path_buf();
                println!("cargo:rustc-link-search=native={}", out_lib_dir.display());
                break;
            }
        }
    }

    println!("cargo:rustc-link-lib=static=wabt");

    // We need to link against C++ std lib
    if let Some(cpp_stdlib) = get_cpp_stdlib() {
        // If a empty library name is specified, then do not link against the stdlib.
        if !cpp_stdlib.is_empty() {
            println!("cargo:rustc-link-lib={}", cpp_stdlib);
        }
    }

    println!("cargo:rerun-if-changed=wabt_shim.cc");
    println!("cargo:rerun-if-changed=wabt/src/emscripten-helpers.cc");

    let mut cfg = cc::Build::new();
    if cfg.get_compiler().is_like_msvc() {
        cfg.flag("/std:c++17");
    } else {
        cfg.flag("-std=c++17");
    }

    cfg.file("wabt/src/emscripten-helpers.cc")
        .file("wabt_shim.cc")
        .include("wabt")
        // This is needed for config.h generated by cmake.
        .include(out_build_dir)
        // We link to stdlib above.
        .cpp_link_stdlib(None)
        .warnings(false)
        .cpp(true)
        .compile("wabt_shim");
}

/// Returns the C++ stdlib to link against specified by an environment variable. If the env vars
/// are not passed, it tries to autodetect.
///
/// The environment variables are `WABT_CXXSTDLIB` and `CXXSTDLIB` (in the priority order). If a
/// variable exists but is empty it is returned as is. In case if a variable is not valid unicode
/// it is skipped.
///
/// Adapted from:
/// https://github.com/alexcrichton/cc-rs/blob/0eeafcc9/src/lib.rs#L2194
fn get_cpp_stdlib() -> Option<String> {
    if let Some(specified_stdlib) = env::var("WABT_CXXSTDLIB")
        .or_else(|_| env::var("CXXSTDLIB"))
        .ok()
    {
        return Some(specified_stdlib);
    }

    env::var("TARGET").ok().and_then(|target| {
        if target.contains("msvc") {
            None
        } else if target.contains("darwin") {
            Some("c++".to_string())
        } else if target.contains("freebsd") {
            Some("c++".to_string())
        } else if target.contains("android") {
            Some("c++".to_string())
        } else if target.contains("musl") {
            Some("static=stdc++".to_string())
        } else {
            Some("stdc++".to_string())
        }
    })
}
