fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    match target_os.as_str() {
        "espidf" => println!("cargo:rustc-cfg=zenoh_pico"),
        "none" => panic!("Unsupported target os: none"),
        _ => {}
    }
}
