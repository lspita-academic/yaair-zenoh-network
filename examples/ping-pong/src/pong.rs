use embassy_time::Timer;
use zenoh_pico::{session::Session, zbytes::TryIntoZBytes};

#[embassy_executor::task]
pub async fn pong(session: &'static Session) {
    log::info!("Starting pong task");
    let publisher = session
        .declare_publisher(
            &"pong/value".parse().expect("Pong keyexpr should be valid"),
            None,
        )
        .expect("Failed to declare pong publisher");
    let subscriber = session
        .declare_subscriber_async(
            &"ping/value".parse().expect("Ping keyexpr should be valid"),
            None,
        )
        .expect("Failed to declare ping subscriber");

    loop {
        log::info!("Waiting ping");
        let sample = subscriber.recv_async().await;
        let bytes = sample.payload().owned_bytes();
        log::info!("Serialized ping size: {}", bytes.len());
        let ping: usize = postcard::from_bytes(&bytes).expect("Failed to deserialize ping");
        log::info!("Received ping: {ping}");

        Timer::after_secs(2).await;

        let pong = ping + 1;
        log::info!("Publishing pong: {pong}");
        let bytes = postcard::to_allocvec(&pong).expect("Failed to serialize pong");
        log::info!("Serialized pong size: {}", bytes.len());
        let payload = bytes
            .try_into_zbytes()
            .expect("Failed to create pong payload");
        publisher
            .put(payload, None)
            .expect("Failed to publish pong");
        log::info!("Published pong");
    }
}
