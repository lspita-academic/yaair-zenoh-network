fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        "espidf" => embuild::espidf::sysenv::output(),
        _ => {}
    }
}
