struct ZenohPicoVar {
    name: &'static str,
    default: usize,
}

// check default values here:
// https://github.com/eclipse-zenoh/zenoh-pico/blob/c2088f198e5c697be1b51bee2916b97f3a9b919f/CMakeLists.txt#L306-L308
const ZENOH_PICO_VARS: [ZenohPicoVar; 3] = [
    ZenohPicoVar {
        name: "ZENOH_PICO_FRAG_MAX_SIZE",
        default: 4096,
    },
    ZenohPicoVar {
        name: "ZENOH_PICO_BATCH_MULTICAST_SIZE",
        default: 2048,
    },
    ZenohPicoVar {
        name: "ZENOH_PICO_BATCH_UNICAST_SIZE",
        default: 2048,
    },
];

fn main() {
    for var in &ZENOH_PICO_VARS {
        println!("cargo:rerun-if-env-changed={}", var.name);

        let value = std::env::var(var.name)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(var.default);

        println!("cargo:rustc-env={}={}", var.name, value);
    }
}
