#![cfg(target_os = "espidf")]

use std::{thread, time::Duration};
use zenoh_pico::{session::Session, zbytes::TryIntoZBytes};

#[cfg(target_os = "espidf")]
#[embassy_executor::task]
pub async fn ping(session: &'static Session) {
    log::info!("Starting ping task");
    let publisher = session
        .declare_publisher(
            &"ping/value".parse().expect("Ping keyexpr should be valid"),
            None,
        )
        .expect("Failed to declare ping publisher");
    let subscriber = session
        .declare_subscriber_async(
            &"pong/value".parse().expect("Pong keyexpr should be valid"),
            None,
        )
        .expect("Failed to declare pong subscriber");

    thread::sleep(Duration::from_secs(2));
    let mut ping = 0usize;
    loop {
        thread::sleep(Duration::from_secs(2));

        log::info!("Publishing ping: {ping}");
        let bytes = postcard::to_allocvec(&ping).expect("Failed to serialize ping");
        log::info!("Serialized ping size: {}", bytes.len());
        let payload = bytes
            .try_into_zbytes()
            .expect("Failed to create ping payload");
        publisher
            .put(payload, None)
            .expect("Failed to publish ping");
        log::info!("Published ping");

        log::info!("Waiting pong");
        let sample = subscriber.recv_async().await;
        let bytes = sample.payload().owned_bytes();
        log::info!("Serialized pong size: {}", bytes.len());
        let pong: usize = postcard::from_bytes(&bytes).expect("Failed to deserialize pong");
        log::info!("Received pong: {pong}");
        ping = pong + 1;
    }
}
