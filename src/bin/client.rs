#[tokio::main]
async fn main() -> anyhow::Result<()> {
    http2_test::client::echo_client().await
}
