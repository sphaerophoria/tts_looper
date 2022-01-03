use std::env;
use std::path::PathBuf;

fn main() -> Result<(), ()> {
    let dst = cmake::Config::new("src/gui/cpp").build();
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=gui");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=Qt5Core");
    println!("cargo:rustc-link-lib=dylib=Qt5Gui");
    println!("cargo:rustc-link-lib=dylib=Qt5Qml");
    println!("cargo:rustc-link-lib=dylib=Qt5Quick");
    println!("cargo:rustc-link-lib=dylib=Qt5QuickControls2");

    let bindings = bindgen::builder().header("src/gui/cpp/gui.h").generate()?;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
