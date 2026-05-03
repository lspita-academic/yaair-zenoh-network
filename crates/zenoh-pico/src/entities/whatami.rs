use std::fmt::Display;

use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use zenoh_pico_sys::{
    z_whatami_t, z_whatami_t_Z_WHATAMI_CLIENT, z_whatami_t_Z_WHATAMI_PEER,
    z_whatami_t_Z_WHATAMI_ROUTER,
};

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    Eq,
    PartialEq,
    EnumCount,
    EnumIter,
    IntoPrimitive,
    TryFromPrimitive,
    UnsafeFromPrimitive,
)]
#[repr(u32)]
pub enum WhatAmI {
    #[default]
    Router = z_whatami_t_Z_WHATAMI_ROUTER,
    Peer = z_whatami_t_Z_WHATAMI_PEER,
    Client = z_whatami_t_Z_WHATAMI_CLIENT,
}

type WhatAmIMaskData = [WhatAmI; WhatAmI::COUNT];

pub struct WhatAmIMask {
    data: WhatAmIMaskData,
    size: usize,
}

impl FromIterator<WhatAmI> for WhatAmIMask {
    fn from_iter<T: IntoIterator<Item = WhatAmI>>(iter: T) -> Self {
        let mut data = WhatAmIMaskData::default();
        let mut size = 0;
        iter.into_iter()
            .take(WhatAmI::COUNT)
            .enumerate()
            .for_each(|(i, w)| {
                data[i] = w;
                size += 1;
            });
        Self { data, size }
    }
}

impl From<z_whatami_t> for WhatAmIMask {
    fn from(value: z_whatami_t) -> Self {
        Self::from_iter(WhatAmI::iter().filter(|w| {
            let whatami_bit: u32 = (*w).into();
            whatami_bit & value == whatami_bit
        }))
    }
}

impl WhatAmIMask {
    pub fn variants(&self) -> &[WhatAmI] {
        &self.data[..self.size]
    }

    pub fn contains(&self, value: &WhatAmI) -> bool {
        self.variants().contains(value)
    }

    pub fn zmask(&self) -> z_whatami_t {
        self.variants()
            .iter()
            .map(|w| (*w).into())
            .reduce(|acc, w| acc | w)
            .unwrap_or(0)
    }
}

impl Display for WhatAmIMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.zmask().to_string())
    }
}
