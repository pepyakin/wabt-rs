extern crate bindgen;
extern crate cmake;
extern crate cc;

fn main() {
    let dst = cmake::Config::new("wabt")
        .define("BUILD_TESTS", "OFF")
        .build();
    println!("cargo:rustc-link-search=native={}/build/", dst.display());
    println!("cargo:rustc-link-lib=static=wabt");

    cc::Build::new()
        .file("wabt/src/emscripten-helpers.cc")
        .compile("emscripten");
}
