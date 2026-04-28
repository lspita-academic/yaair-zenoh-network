use zenoh_pico_macros::{zclosure, zown};

#[zown(base = "sample", zloan(mutable), ztake)]
pub struct Sample;

#[zclosure(base = "sample", zloan)]
pub struct SampleClosure;
