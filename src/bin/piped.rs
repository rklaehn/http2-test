use bytes::Bytes;
use flume::{Receiver, Sender};
use futures::TryStreamExt;
use futures::{future::FutureExt, stream::StreamExt};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

#[derive(Clone)]
struct EchoServer {
    tx: Sender<(Receiver<Bytes>, Sender<Bytes>)>,
}

impl EchoServer {
    async fn echo(self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        // Create Flume channels for the request body and response body
        let (req_tx, req_rx) = flume::bounded(1);
        let (res_tx, res_rx) = flume::bounded(1);
        let _ = self.tx.send((req_rx, res_tx));
        tokio::task::spawn(async move {
        req.into_body()
            .try_for_each(|chunk| async {
                req_tx.send_async(chunk).await.unwrap();
                Ok(())
            })
            .await?;
            anyhow::Ok(())
        });
        let body = res_rx.into_stream().map(anyhow::Ok);
        // Create a response with the response body channel as the response body
        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::wrap_stream(body))
            .unwrap();

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a global Flume channel for publishing request/response pairs
    let (tx, rx) = flume::bounded(1);

    // Create an EchoServer instance with the global channel
    let echo_server = EchoServer { tx };

    // Create a service that uses the EchoServer to handle requests
    let service = make_service_fn(move |_| {
        let es = echo_server.clone();
        async move { Ok::<_, hyper::Error>(service_fn(move |req| es.clone().echo(req))) }
    });

    // Create a server from the service
    let server = Server::bind(&([127, 0, 0, 1], 3000).into())
        .http2_only(true)
        .http2_max_frame_size(1024 * 1024 * 2)
        .http2_max_send_buf_size(1024 * 1024 * 2)
        .serve(service);
    tokio::task::spawn(async move {
        while let Some((a, b)) = rx.stream().next().await {
            println!("got request!");
            tokio::task::spawn(async move {
                while let Ok(chunk) = a.recv_async().await {
                    println!("got chunk! {}", chunk.len());
                    let res = Bytes::from(vec![0; 8]);
                    b.send_async(res).await.unwrap();
                }
            });
        }
    });

    // Start the server and use the `select` combinator to run it indefinitely
    server.await?;

    Ok(())
}
