use std::env;
use std::path::{Path, PathBuf};

fn copy_file(src: &Path, dst: &Path) {
    if dst.exists() {
        std::fs::remove_file(&dst).unwrap();
    }

    std::fs::copy(src, dst).unwrap();
}

#[cfg(target_os = "linux")]
fn copy_lib_to_out_dir(manifest_dir: &Path, out_dir: &Path) {
    copy_file(
        &manifest_dir.join("res/linux/libdeepspeech.so"),
        &out_dir.join("libdeepspeech.so"),
    );
}

#[cfg(target_os = "windows")]
fn copy_lib_to_out_dir(manifest_dir: &Path, out_dir: &Path) {
    // Need the .so at runtime and the .dll at link time. Rust can only use -l
    // flags to link the so, and it won't find lib*.so on windows. Unfortunately
    // at runtime we still expect to find the .so even if we linked against the
    // renamed dll. 
    let output_so = out_dir.join("libdeepspeech.so");
    let output_dll = out_dir.join("libdeepspeech.dll");
    copy_file(
        &manifest_dir.join("res/windows/libdeepspeech.so"),
        &output_so,
    );
    if output_dll.exists() {
        std::fs::remove_file(&output_dll).unwrap();
    }
    std::fs::hard_link(output_so, output_dll).unwrap();
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:rustc-link-lib=dylib=deepspeech");

    let bindings = bindgen::builder()
        .header(&manifest_dir.join("res/deepspeech.h").display().to_string())
        .generate()
        .unwrap();

    bindings
        .write_to_file(format!("{}/bindings.rs", out_dir))
        .unwrap();

    // Cargo will append link search paths to LD_LIBRARY_PATH/PATH, but only if
    // they're in OUT_DIR. Copy the lib there so that cargo run works with no
    // configuration
    copy_lib_to_out_dir(&manifest_dir, &PathBuf::from(&out_dir));
    println!("cargo:rustc-link-search={}", out_dir);
}
