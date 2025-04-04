use futures::join;
use wasi::async_support::spawn;
use wasi::http::types::{ErrorCode, Headers, Request, Response};
use wasi::{wit_future, wit_stream};

struct Component;

wasi::http::proxy::export!(Component);

impl wasi::exports::http::handler::Guest for Component {
    async fn handle(request: Request) -> Result<Response, ErrorCode> {
        let (mut stderr_tx, stderr_rx) = wit_stream::new();
        wasi::cli::stderr::set_stderr(stderr_rx);

        let (mut contents_tx, contents_rx) = wit_stream::new();
        let (trailers_tx, trailers_rx) = wit_future::new();
        let (resp, transmit) = Response::new(Headers::new(), Some(contents_rx), trailers_rx);

        let pq = request.path_with_query();
        let pq = pq.as_deref().unwrap_or("/");
        stderr_tx
            .write(format!("\nServing request for {pq}\n").into())
            .await;
        let method = request.method();
        stderr_tx
            .write(format!("Method: {method:?}\n").into())
            .await;
        let headers = request.headers().entries();
        if !headers.is_empty() {
            stderr_tx.write("\nHeaders:\n".into()).await;
            for (k, v) in headers {
                stderr_tx
                    .write(format!("{k}: {}\n", String::from_utf8_lossy(&v)).into())
                    .await;
            }
        }

        spawn(async {
            join!(
                async {
                    let remaining = contents_tx.write_all(b"hello, world!".to_vec()).await;
                    assert!(remaining.is_empty());
                    drop(contents_tx);
                    trailers_tx
                        .write(Ok(None))
                        .await
                        .expect("failed to write trailers");
                },
                async {
                    transmit
                        .await
                        .expect("failed to transmit response")
                        .unwrap()
                }
            );
        });
        Ok(resp)
    }
}
