use std::{
    fmt::{self, Display},
    hash::Hash,
};

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
    ser::SerializeStruct,
};
use uuid::Uuid;
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
}

impl From<ZIdBytes> for ZId {
    fn from(value: ZIdBytes) -> Self {
        Self::from_zvalue(_z_id_t { id: value })
    }
}

impl From<Uuid> for ZId {
    fn from(value: Uuid) -> Self {
        Self::from(value.into_bytes())
    }
}

impl From<ZId> for Uuid {
    fn from(value: ZId) -> Self {
        Self::from_bytes(value.id)
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

impl Serialize for ZId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("z_id_t", 1)?;
        s.serialize_field("id", &self.0.id)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for ZId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Id,
        }

        struct ZIdVisitor;

        impl<'de> Visitor<'de> for ZIdVisitor {
            type Value = ZId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ZId")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let id: ZIdBytes = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(id.into())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut id = None::<ZIdBytes>;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                Ok(id.into())
            }
        }

        deserializer.deserialize_struct("ZId", &["id"], ZIdVisitor)
    }
}
