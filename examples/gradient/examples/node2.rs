use embassy_executor::Spawner;
use gradient_example::Node;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    gradient_example::gradient_main(Node::Node2, spawner).await;
}
