#[tokio::main]
async fn main() -> anyhow::Result<()> {
    http2_test::server::echo_server().await
}
