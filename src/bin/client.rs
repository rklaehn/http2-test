use std::any;
use std::time::Instant;

use bytes::Bytes;
use futures::{Stream, StreamExt, FutureExt};
use hyper::body::HttpBody;
use hyper::{Body, Client, Request, Response, Uri};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn client(url: Uri, stream: impl Stream<Item = Bytes> + Send + 'static) -> anyhow::Result<impl Stream<Item = std::result::Result<Bytes, hyper::Error>>> {
    let stream = stream.map(anyhow::Ok);
    let req = Request::post(url).body(Body::wrap_stream(stream))?;

    let client = Client::builder()
        .http2_only(true)
        // .http2_initial_connection_window_size(Some(1024 * 1024 *2))
        // .http2_initial_stream_window_size(Some(1024 * 1024 *2))
        .http2_max_frame_size(1024 * 1024 * 2)
        .http2_max_send_buf_size(1024 * 1024 * 2)
        .build_http();
    // Send the request and wait for the response
    let res = client.request(req).await?;

    let stream = futures::stream::unfold(res, move |mut res| {
        async move {
            let chunk = res.body_mut().data().await?;
            Some((chunk, res))
        }
    });

    Ok(stream)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let block_size = 1024 * 1024;
    let block_count = 1000;
    let url = "http://127.0.0.1:3000".parse()?;
    let blocks = (0..block_count).map(move |i| Bytes::from(vec![i as u8; block_size]));
    let blocks = futures::stream::iter(blocks);
    let t0 = Instant::now();
    let response = client(url, blocks).await?;
    tokio::pin!(response);
    let sent = block_size * block_count;
    let mut received: u64 = 0;
    while let Some(Ok(chunk)) = response.next().await {
        received += chunk.len() as u64;
        println!("{}", chunk.len());
    }
    let elapsed = t0.elapsed().as_secs_f64();
    println!("elapsed {}", elapsed);
    println!("send rate {}", ((sent as f64) / elapsed) as u64);
    println!("recv rate {}", ((received as f64) / elapsed) as u64);
    Ok(())
}
