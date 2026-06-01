#[cfg(target_os = "espidf")]
pub mod esp;

#[cfg(target_os = "espidf")]
pub fn init() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::init_from_env();
}

#[cfg(not(target_os = "espidf"))]
pub fn init() {
    env_logger::init();
}
