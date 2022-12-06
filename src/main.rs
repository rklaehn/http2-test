use futures::TryStreamExt;
use futures::{future::FutureExt, stream::StreamExt};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let chunks = req.into_body().map_ok(|chunk| {
        // Print each chunk of the request body
        // println!("chunk: {:?}", chunk.len());
        chunk
    });

    // Create a response with the request body as the response body
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::wrap_stream(chunks))
        .unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a service that echoes the request body back as the response body
    let service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(echo))
    });

    // Create a server from the service
    let server = Server::bind(&([127, 0, 0, 1], 3000).into())
        .serve(service);

    // Start the server and use the `select` combinator to run it indefinitely
    server.await?;

    Ok(())
}
