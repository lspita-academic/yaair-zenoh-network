pub const fn frag_max_size_str() -> &'static str {
    env!("ZENOH_PICO_FRAG_MAX_SIZE")
}

fn parse_into_usize(value: &str) -> usize {
    value.parse().expect("Should be a valid usize")
}

pub fn frag_max_size() -> usize {
    parse_into_usize(frag_max_size_str())
}

pub const fn batch_unicast_size_str() -> &'static str {
    env!("ZENOH_PICO_BATCH_UNICAST_SIZE")
}

pub fn batch_unicast_size() -> usize {
    parse_into_usize(batch_unicast_size_str())
}

pub const fn batch_multicast_size_str() -> &'static str {
    env!("ZENOH_PICO_BATCH_MULTICAST_SIZE")
}

pub fn batch_multicast_size() -> usize {
    parse_into_usize(batch_multicast_size_str())
}
