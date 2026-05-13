use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    gradient::gradient_main(gradient::Node::Node2, spawner).await;
}
