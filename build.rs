use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("TARGET")?.as_str() != "x86_64-pc-windows-gnullvm" {
        return Ok(());
    }

    // Rust's GNU toolchain includes the Win32 import libraries used by eframe.
    let gnu_libraries = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("target/rustup/toolchains/1.97.1-x86_64-pc-windows-gnu")
        .join("lib/rustlib/x86_64-pc-windows-gnu/lib/self-contained");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for library in ["advapi32", "gdi32", "msimg32", "opengl32", "shell32", "winspool"] {
        let file = format!("lib{library}.a");
        let source = gnu_libraries.join(&file);
        println!("cargo::rerun-if-changed={}", source.display());
        fs::copy(&source, output.join(&file)).map_err(|err| {
            format!("{}: {err}. Run .cargo\\rustc-windows.cmd to install the Windows build libraries.", source.display())
        })?;
    }
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-link-search=native={}", output.display());
    Ok(())
}
