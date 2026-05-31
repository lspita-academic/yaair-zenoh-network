use itertools::Itertools;
use strum::{Display, EnumString, VariantNames};
use thiserror::Error;

#[derive(EnumString, VariantNames, Display)]
#[strum(serialize_all = "snake_case")]
enum ZenohImpl {
    Zenoh,
    ZenohPico,
}

#[derive(Debug, Error)]
enum ZenohError {
    #[error("Unsupported target os: {0}")]
    UnsupportedTargetOs(String),
}

impl ZenohImpl {
    fn declare_cfg() {
        let zenoh_impl_values = ZenohImpl::VARIANTS
            .iter()
            .format_with(", ", |s, f| f(&format_args!("\"{s}\"")));
        println!("cargo::rustc-check-cfg=cfg(zenoh_impl, values({zenoh_impl_values}))");
    }

    fn detect_from_os(target_os: &str) -> Result<Self, ZenohError> {
        match target_os {
            "espidf" => Ok(Self::ZenohPico),
            "none" => Err(ZenohError::UnsupportedTargetOs("none".to_owned())),
            _ => Ok(Self::Zenoh),
        }
    }

    fn set_cfg(&self) {
        println!("cargo:rustc-cfg=zenoh_impl=\"{self}\"");
    }
}

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    ZenohImpl::declare_cfg();
    ZenohImpl::detect_from_os(&target_os)
        .expect("Error selecting zenoh implementation")
        .set_cfg();
}
