use std::time::Duration;

use embassy_time::Timer;
use yaair_zenoh_network::HeartbitPublisher;

use crate::{EmbassyDuration, Serializer};

#[embassy_executor::task]
pub async fn periodic_heartbit_task(
    heartbit_publisher: HeartbitPublisher<Serializer>,
    period: EmbassyDuration,
    lifespan: Option<Duration>,
) {
    log::warn!("Heartbit task started");

    if let Some(lifespan) = lifespan {
        log::warn!("Publishing lifespan [ms]: {}", lifespan.as_millis());
        heartbit_publisher.put_lifespan(lifespan);
    }

    loop {
        log::warn!("Publishing keep alive");
        heartbit_publisher.put_keep_alive();
        Timer::after(period).await;
    }
}
