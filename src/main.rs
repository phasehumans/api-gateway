mod engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    engine::run().await
}
