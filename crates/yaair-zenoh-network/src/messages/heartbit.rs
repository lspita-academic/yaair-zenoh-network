use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Heartbit<Sender> {
    pub sender: Sender,
    pub lifespan: Option<Duration>,
}
