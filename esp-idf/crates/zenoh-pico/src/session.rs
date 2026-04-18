use std::ffi::c_void;

use esp_idf_svc::sys::zenoh_pico::{
    _z_res_t_Z_OK, z_close, z_closure_zid, z_closure_zid_callback_t, z_closure_zid_move,
    z_config_move, z_id_t, z_info_peers_zid, z_open, z_open_options_t, z_owned_closure_zid_t,
    z_owned_session_t, z_session_drop, z_session_is_closed, z_session_loan, z_session_loan_mut,
    z_session_move, zp_start_lease_task, zp_start_read_task,
};

use super::{config::ZenohConfig, publisher::ZenohPublisher, subscriber::ZenohSubscriber};

pub struct ZenohSession {
    pub(super) z_session: z_owned_session_t,
}

impl From<z_owned_session_t> for ZenohSession {
    fn from(z_session: z_owned_session_t) -> Self {
        Self { z_session }
    }
}

impl Drop for ZenohSession {
    fn drop(&mut self) {
        self.close();
        let z_session = &mut self.z_session;
        unsafe {
            z_session_drop(z_session_move(z_session));
        }
    }
}

impl ZenohSession {
    pub fn open(mut config: ZenohConfig, z_open_options: Option<z_open_options_t>) -> Self {
        let z_config = &mut config.z_config;
        let mut z_session = z_owned_session_t::default();
        let open_options = z_open_options
            .map(|o| &o as *const z_open_options_t)
            .unwrap_or(std::ptr::null());

        let result = unsafe { z_open(&mut z_session, z_config_move(z_config), open_options) };
        assert!(
            result == _z_res_t_Z_OK as i8, // crash if no scouts found
            "Cannot open zenoh session: {}",
            result
        );
        unsafe {
            // not done automatically, even if it should be because of the default options
            zp_start_read_task(z_session_loan_mut(&mut z_session), std::ptr::null());
            zp_start_lease_task(z_session_loan_mut(&mut z_session), std::ptr::null());
        }

        Self::from(z_session)
    }

    pub fn close(&mut self) {
        let z_session = &mut self.z_session;
        let z_session_closed = unsafe { z_session_is_closed(z_session_loan(z_session)) };
        if !z_session_closed {
            let close_options = std::ptr::null();
            unsafe {
                z_close(z_session_loan_mut(z_session), close_options);
            }
        }
    }

    pub fn print_peers_zid(&self) {
        log::info!("Printing peers:");
        let z_session = &self.z_session;
        let closure_callback: z_closure_zid_callback_t = Some(print_peer);
        let mut closure_zid = z_owned_closure_zid_t::default();
        unsafe {
            z_closure_zid(
                &mut closure_zid,
                closure_callback,
                None,
                std::ptr::null_mut(),
            );
            z_info_peers_zid(
                z_session_loan(z_session),
                z_closure_zid_move(&mut closure_zid),
            );
        }
    }

    pub fn publisher(&self, key: &str) -> ZenohPublisher {
        ZenohPublisher::new(self, key)
    }

    pub fn subscriber(&self, key: &str) -> ZenohSubscriber {
        ZenohSubscriber::new(self, key)
    }
}

unsafe extern "C" fn print_peer(id: *const z_id_t, _: *mut c_void) {
    let bytes = unsafe { (*id).id };
    let z_id = bytes.map(|b| b.to_string()).join("");
    log::info!("Peer: {}", z_id);
}
