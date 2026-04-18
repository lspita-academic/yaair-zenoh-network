use std::ffi::CStr;

use super::session::ZenohSession;
use esp_idf_svc::sys::zenoh_pico::{
    z_bytes_from_string, z_bytes_move, z_declare_publisher, z_keyexpr_drop, z_keyexpr_from_str,
    z_keyexpr_loan, z_keyexpr_move, z_owned_bytes_t, z_owned_keyexpr_t, z_owned_publisher_t,
    z_owned_string_t, z_publisher_drop, z_publisher_loan, z_publisher_move,
    z_publisher_options_default, z_publisher_options_t, z_publisher_put,
    z_publisher_put_options_default, z_publisher_put_options_t, z_session_loan,
    z_string_copy_from_str, z_string_move,
};

pub struct ZenohPublisher {
    pub(super) z_publisher: z_owned_publisher_t,
    pub(super) z_keyexpr: z_owned_keyexpr_t,
    key: String,
}

impl ZenohPublisher {
    pub fn new(session: &ZenohSession, key: &str) -> Self {
        let z_session = &session.z_session;
        let mut z_keyexpr = z_owned_keyexpr_t::default();
        let key_bytes = [key.as_bytes(), &[0]].concat();
        let key_cstr = CStr::from_bytes_until_nul(&key_bytes).unwrap();

        let mut z_publisher = z_owned_publisher_t::default();
        let mut options = z_publisher_options_t::default();
        unsafe {
            z_publisher_options_default(&mut options);
            z_keyexpr_from_str(&mut z_keyexpr, key_cstr.as_ptr());
            z_declare_publisher(
                z_session_loan(z_session),
                &mut z_publisher,
                z_keyexpr_loan(&z_keyexpr),
                &options,
            );
        };
        Self {
            z_publisher,
            z_keyexpr,
            key: key.to_owned(),
        }
    }

    pub fn put(&self, value: &str) {
        let value_bytes = [value.as_bytes(), &[0]].concat();
        let value_cstr = CStr::from_bytes_until_nul(&value_bytes).unwrap();
        let mut value_z_string = z_owned_string_t::default();
        let mut value_z_bytes = z_owned_bytes_t::default();
        let mut options = z_publisher_put_options_t::default();
        log::info!("Publishing on {}: {}", self.key, value);
        unsafe {
            z_publisher_put_options_default(&mut options);
            z_string_copy_from_str(&mut value_z_string, value_cstr.as_ptr());
            z_bytes_from_string(&mut value_z_bytes, z_string_move(&mut value_z_string));
            z_publisher_put(
                z_publisher_loan(&self.z_publisher),
                z_bytes_move(&mut value_z_bytes),
                &options,
            );
        }
    }
}

impl Drop for ZenohPublisher {
    fn drop(&mut self) {
        unsafe {
            z_keyexpr_drop(z_keyexpr_move(&mut self.z_keyexpr));
            z_publisher_drop(z_publisher_move(&mut self.z_publisher));
        }
    }
}
