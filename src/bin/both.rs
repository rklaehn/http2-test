use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let handle = tokio::task::spawn(http2_test::server::echo_server());
    tokio::time::sleep(Duration::from_secs(1)).await;
    http2_test::client::echo_client().await?;
    handle.abort();
    let _ = handle.await;
    Ok(())
}
