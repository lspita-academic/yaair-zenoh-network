use std::time::Duration;

use embassy_time::Timer;
use yaair_zenoh_network::heartbeat::HeartbeatPublisher;

use crate::{EmbassyDuration, Serializer};

#[embassy_executor::task]
pub async fn periodic_heartbeat_task(
    heartbeat_publisher: HeartbeatPublisher<Serializer>,
    period: EmbassyDuration,
    lifespan: Option<Duration>,
) {
    log::warn!("Heartbeat task started");

    if let Some(lifespan) = lifespan {
        log::warn!("Publishing lifespan [ms]: {}", lifespan.as_millis());
        heartbeat_publisher.put_lifespan(lifespan);
    }

    loop {
        log::warn!("Publishing keep alive");
        heartbeat_publisher.put_keep_alive();
        Timer::after(period).await;
    }
}
