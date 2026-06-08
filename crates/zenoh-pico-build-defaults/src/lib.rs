pub const fn frag_max_size_str() -> &'static str {
    env!("ZENOH_PICO_FRAG_MAX_SIZE")
}

pub const fn batch_unicast_size_str() -> &'static str {
    env!("ZENOH_PICO_BATCH_UNICAST_SIZE")
}

pub const fn batch_multicast_size_str() -> &'static str {
    env!("ZENOH_PICO_BATCH_MULTICAST_SIZE")
}
