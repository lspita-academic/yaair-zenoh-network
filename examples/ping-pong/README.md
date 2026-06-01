# Ping-pong example

This is an example for the [zenoh pico wrapper](../../crates/zenoh-pico). There
are 2 different nodes, represented by corresponding examples, that share a common
"main" logic (located in [the crate lib](./src/lib.rs) and it's modules).

> [!warning] Cross-compilation
> This example only supports targets of the `espidf` family, like the zenoh pico
> wrapper itself.

The example consists in 2 nodes using zenoh pico to send each other a number,
increasing it every round trip. It uses the async/await pattern instead of a callback
based one for a cleaner API.
