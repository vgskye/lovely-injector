fn main() {
    #[cfg(target_os = "windows")]
    forward_dll::forward_dll("C:\\Windows\\System32\\version.dll").unwrap();
    println!("cargo:rustc-link-lib=dylib=lua51");
}
