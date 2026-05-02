use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohResult},
    sys::{
        z_congestion_control_t_Z_CONGESTION_CONTROL_BLOCK,
        z_congestion_control_t_Z_CONGESTION_CONTROL_DROP, z_declare_publisher,
        z_priority_t__Z_PRIORITY_CONTROL, z_priority_t_Z_PRIORITY_BACKGROUND,
        z_priority_t_Z_PRIORITY_DATA, z_priority_t_Z_PRIORITY_DATA_HIGH,
        z_priority_t_Z_PRIORITY_DATA_LOW, z_priority_t_Z_PRIORITY_INTERACTIVE_HIGH,
        z_priority_t_Z_PRIORITY_INTERACTIVE_LOW, z_priority_t_Z_PRIORITY_REAL_TIME,
        z_publisher_options_default, z_publisher_options_t, z_undeclare_publisher,
    },
    zvalue::ZValue,
};
use zenoh_pico_macros::zwrap;

use crate::{
    keyexpr::KeyExpr,
    session::Session,
    zoptions::{ZOptionsInit, options_ptr},
};

impl ZOptionsInit for z_publisher_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_publisher_options_default(self);
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
    pub fn declare(
        session: &Session,
        key: &KeyExpr,
        publisher_options: Option<z_publisher_options_t>,
    ) -> ZenohResult<Self> {
        let publisher_options = options_ptr(publisher_options.as_ref());
        let mut publisher = Default::default();
        unsafe {
            z_declare_publisher(
                session.zloan(),
                &mut publisher,
                key.zloan(),
                publisher_options,
            )
            .into_zresult()?;
        };

        Ok(Self::from(publisher))
    }
}
