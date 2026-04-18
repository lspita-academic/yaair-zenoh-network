use std::ffi::{CStr, c_void};

use super::session::ZenohSession;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use esp_idf_svc::sys::zenoh_pico::{
    _z_sample_t, z_bytes_to_string, z_closure_sample, z_closure_sample_callback_t,
    z_closure_sample_move, z_declare_subscriber, z_keyexpr_drop, z_keyexpr_from_str,
    z_keyexpr_loan, z_keyexpr_move, z_owned_closure_sample_t, z_owned_keyexpr_t, z_owned_string_t,
    z_owned_subscriber_t, z_sample_payload, z_session_loan, z_string_data, z_string_len,
    z_string_loan, z_subscriber_drop, z_subscriber_move, z_subscriber_options_default,
    z_subscriber_options_t,
};

pub struct ZenohSubscriber {
    pub(super) z_subscriber: z_owned_subscriber_t,
    pub(super) z_keyexpr: z_owned_keyexpr_t,
    key: String,
    signal: Box<Signal<CriticalSectionRawMutex, String>>,
}

impl ZenohSubscriber {
    pub fn new(session: &ZenohSession, key: &str) -> Self {
        let z_session = &session.z_session;
        let mut z_keyexpr = z_owned_keyexpr_t::default();
        let key_bytes = [key.as_bytes(), &[0]].concat();
        let key_cstr = CStr::from_bytes_until_nul(&key_bytes).unwrap();

        let mut closure_sample = z_owned_closure_sample_t::default();
        let callback: z_closure_sample_callback_t = Some(sub_key_callback);
        let mut signal = Box::new(Signal::new());

        let mut z_subscriber = z_owned_subscriber_t::default();
        let mut options = z_subscriber_options_t::default();
        unsafe {
            z_subscriber_options_default(&mut options);
            z_closure_sample(
                &mut closure_sample,
                callback,
                None,
                signal.as_mut() as *mut Signal<CriticalSectionRawMutex, String> as *mut c_void,
            );
            z_keyexpr_from_str(&mut z_keyexpr, key_cstr.as_ptr());
            z_declare_subscriber(
                z_session_loan(z_session),
                &mut z_subscriber,
                z_keyexpr_loan(&z_keyexpr),
                z_closure_sample_move(&mut closure_sample),
                &options,
            );
        };
        Self {
            z_subscriber,
            z_keyexpr,
            key: key.to_owned(),
            signal,
        }
    }

    pub async fn recv_async(&self) -> String {
        log::info!("Waiting for signal on {}", self.key);
        let value = self.signal.wait().await;
        log::info!("Received {}: {}", self.key, value);
        self.signal.reset();
        value
    }
}

impl Drop for ZenohSubscriber {
    fn drop(&mut self) {
        unsafe {
            z_keyexpr_drop(z_keyexpr_move(&mut self.z_keyexpr));
            z_subscriber_drop(z_subscriber_move(&mut self.z_subscriber));
        }
    }
}

unsafe extern "C" fn sub_key_callback(sample: *mut _z_sample_t, context: *mut c_void) {
    let signal = unsafe { &mut *(context as *mut Signal<CriticalSectionRawMutex, String>) };
    let payload = unsafe { z_sample_payload(sample) };
    let mut payload_buffer = z_owned_string_t::default();
    let payload_string_bytes;
    let value = unsafe {
        z_bytes_to_string(payload, &mut payload_buffer);
        let payload_z_raw = z_string_data(z_string_loan(&payload_buffer));
        let payload_len = z_string_len(z_string_loan(&payload_buffer));
        payload_string_bytes =
            [std::slice::from_raw_parts(payload_z_raw, payload_len), &[0]].concat();
        CStr::from_bytes_until_nul(&payload_string_bytes)
            .unwrap()
            .to_str()
            .unwrap()
    };
    signal.signal(value.to_owned());
}
