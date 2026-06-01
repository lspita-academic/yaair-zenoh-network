# yaair-zenoh-network

A [YAAIR](https://github.com/nicolasfara/yaair) network implementation using the 
Zenoh protocol.

This is a project created for my [bachelor's degree thesis](https://github.com/lspita-academic/unibo-thesis) in Computer Science and Engineering at University of Bologna.

# Installation

The main crate is [`yaair-zenoh-network`](./crates/yaair-zenoh-network), that provides
everything necessary to work.
If you also want to directly access zenoh pico directly (e.g. for extra
configuration options) the [`zenoh-pico`](./crates/zenoh-pico) crate provides a incomplete
wrapper that covers just the things used in this library.

> [!important] ESP-IDF Components
> `yaair-zenoh-network` defines some extra esp-idf components (to compile zenoh pico
as a dependency): `esp-idf-sys` automatically detects components from direct dependencies
of the crate, but in the case of a workspace they must be dependencies of the configuration
crate denoted by the `ESP_IDF_SYS_ROOT_CRATE` env variable (see: <https://github.com/esp-rs/esp-idf-sys/blob/master/BUILD-OPTIONS.md#esp-idf-configuration>).

# Usage

See the [gradient example](./examples/gradient/)
