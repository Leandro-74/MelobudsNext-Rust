//! Embeds the application icon in the Windows executables.
//!
//! The build script only runs for Windows targets. On Linux, `cargo-xwin` uses
//! `llvm-rc`; on Windows, `winresource` locates `rc.exe` from the MSVC
//! toolchain.

fn main() {
    println!("cargo:rerun-if-changed=assets/melobudsnext.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("assets/melobudsnext.ico");
    resource
        .compile()
        .expect("failed to embed the Windows icon");
}
