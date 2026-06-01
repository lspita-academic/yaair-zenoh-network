use std::{
    fmt::{self, Display},
    hash::Hash,
};

use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{_z_id_cmp, _z_id_hash, _z_id_t, ZENOH_ID_SIZE, z_id_to_string};

use crate::{
    result::IntoZenohResult,
    zstring::ZString,
    zvalue::{ZOwn, ZValue},
};

#[zwrap(base(name = "id"), zvalue, zclone)]
#[derive(Debug, Copy)]
pub struct ZId;

pub type ZIdBytes = [u8; ZId::SIZE];

impl ZId {
    pub const SIZE: usize = ZENOH_ID_SIZE as usize;

    pub fn as_bytes(&self) -> &ZIdBytes {
        &self.0.id
    }

    pub fn into_bytes(&self) -> ZIdBytes {
        self.0.id
    }
}

impl From<ZIdBytes> for ZId {
    fn from(value: ZIdBytes) -> Self {
        Self::from_zvalue(_z_id_t { id: value })
    }
}

#[zwrap(base(name = "closure_zid"), zvalue, zown, zclosure(callback_ty = <ZId as ZValue>::Value))]
pub struct ZIdClosure;

impl Display for ZId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = ZString::uninitialized();
        string
            .with_zowned_mut(|z| unsafe { z_id_to_string(&self.0, z).into_zresult() })
            .map_err(|_| fmt::Error)
            .and_then(|_| string.fmt(f))
    }
}

impl Hash for ZId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(unsafe { _z_id_hash(self.zloan()) });
    }
}

impl Eq for ZId {}
impl PartialEq for ZId {
    fn eq(&self, other: &Self) -> bool {
        // https://github.com/eclipse-zenoh/zenoh-pico/blob/3b3ab65cadbb10a8d7f32ba04cb15c26b8435dd5/include/zenoh-pico/protocol/core.h#L74
        self.0.id == other.0.id
    }
}

impl Ord for ZId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe { _z_id_cmp(self.zloan(), other.zloan()) }.cmp(&0)
    }
}

impl PartialOrd for ZId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
