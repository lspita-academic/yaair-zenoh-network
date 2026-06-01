# Gradient example

This is an example for the [yaair zenoh network](../../crates/yaair-zenoh-network). 
There are 3 different nodes, represented by corresponding examples, that share a 
common "main" logic (located in [the crate lib](./src/lib.rs) and it's modules).

The example consists in using aggregate computing to calculate the distance of each
node from the source node. The distances, connections and sources are fixed for
demonstration purposes.
