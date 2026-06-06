use std::{borrow::Borrow, collections::BTreeSet, fmt::Display};

use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use strum::{EnumIter, IntoEnumIterator};
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
    EnumIter,
    PartialEq,
    IntoPrimitive,
    TryFromPrimitive,
    UnsafeFromPrimitive,
    PartialOrd,
    Ord,
)]
#[repr(u32)]
pub enum WhatAmI {
    #[default]
    Router = z_whatami_t_Z_WHATAMI_ROUTER,
    Peer = z_whatami_t_Z_WHATAMI_PEER,
    Client = z_whatami_t_Z_WHATAMI_CLIENT,
}

#[derive(Default)]
pub struct WhatAmIMask(BTreeSet<WhatAmI>);

impl WhatAmIMask {
    pub fn add(&mut self, what: WhatAmI) {
        self.0.insert(what);
    }

    pub fn remove<Q: Borrow<WhatAmI>>(&mut self, what: Q) {
        self.0.remove(what.borrow());
    }

    pub fn contains(&self, value: &WhatAmI) -> bool {
        self.0.contains(value)
    }

    pub fn zmask(&self) -> z_whatami_t {
        self.0
            .iter()
            .map(|w| (*w).into())
            .reduce(|acc, w| acc | w)
            .unwrap_or(0)
    }
}

impl FromIterator<WhatAmI> for WhatAmIMask {
    fn from_iter<T: IntoIterator<Item = WhatAmI>>(iter: T) -> Self {
        let data = iter.into_iter().collect();
        Self(data)
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

impl Display for WhatAmIMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.zmask().to_string())
    }
}
