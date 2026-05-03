use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    z_congestion_control_t_Z_CONGESTION_CONTROL_BLOCK,
    z_congestion_control_t_Z_CONGESTION_CONTROL_DROP, z_priority_t__Z_PRIORITY_CONTROL,
    z_priority_t_Z_PRIORITY_BACKGROUND, z_priority_t_Z_PRIORITY_DATA,
    z_priority_t_Z_PRIORITY_DATA_HIGH, z_priority_t_Z_PRIORITY_DATA_LOW,
    z_priority_t_Z_PRIORITY_INTERACTIVE_HIGH, z_priority_t_Z_PRIORITY_INTERACTIVE_LOW,
    z_priority_t_Z_PRIORITY_REAL_TIME, z_publisher_keyexpr, z_publisher_options_default,
    z_publisher_options_t, z_publisher_put, z_publisher_put_options_default,
    z_publisher_put_options_t, z_undeclare_publisher,
};

use crate::{
    keyexpr::KeyExpr,
    result::{IntoZenohResult, ZenohResult},
    zbytes::IntoZBytes,
    zoptions::{ZOptionsInit, options_ptr},
    zvalue::{ZOwn, ZValue},
};

impl ZOptionsInit for z_publisher_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_publisher_options_default(self);
        }
    }
}

impl ZOptionsInit for z_publisher_put_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_publisher_put_options_default(self);
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(u32)]
pub enum MessagePriority {
    Control = z_priority_t__Z_PRIORITY_CONTROL,
    Realtime = z_priority_t_Z_PRIORITY_REAL_TIME,
    InteractiveHigh = z_priority_t_Z_PRIORITY_INTERACTIVE_HIGH,
    InteractiveLow = z_priority_t_Z_PRIORITY_INTERACTIVE_LOW,
    DataHigh = z_priority_t_Z_PRIORITY_DATA_HIGH,
    #[default]
    Data = z_priority_t_Z_PRIORITY_DATA,
    DataLow = z_priority_t_Z_PRIORITY_DATA_LOW,
    Background = z_priority_t_Z_PRIORITY_BACKGROUND,
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(u32)]
pub enum CongestionControl {
    Block = z_congestion_control_t_Z_CONGESTION_CONTROL_BLOCK,
    #[default]
    Drop = z_congestion_control_t_Z_CONGESTION_CONTROL_DROP,
}

#[zwrap(base(name = "publisher"), zvalue, zown(drop_zfn = z_undeclare_publisher))]
pub struct Publisher;

impl Publisher {
    pub fn put<V: IntoZBytes>(
        &self,
        value: V,
        put_options: Option<z_publisher_put_options_t>,
    ) -> ZenohResult<()> {
        let put_options = options_ptr(put_options.as_ref());
        let payload = value.into_zbytes();
        unsafe { z_publisher_put(self.zloan(), &mut payload.zmove(), put_options).into_zresult() }
    }

    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_publisher_keyexpr(self.zloan()) })
    }
}
