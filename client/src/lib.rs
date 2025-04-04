use futures::join;
use wasi::http::types::{Headers, Request};
use wasi::http::{handler::handle, types::Scheme};
use wasi::{wit_future, wit_stream};

struct Component;

wasi::cli::command::export!(Component);

impl wasi::exports::cli::run::Guest for Component {
    async fn run() -> Result<(), ()> {
        let (mut stdout_tx, stdout_rx) = wit_stream::new();
        wasi::cli::stdout::set_stdout(stdout_rx);

        let (mut stderr_tx, stderr_rx) = wit_stream::new();
        wasi::cli::stderr::set_stderr(stderr_rx);

        let args = wasi::cli::environment::get_arguments();
        let (port, pq) = match args.as_slice() {
            [_, port, pq] => (port.parse().expect("failed to parse port"), Some(pq)),
            [_, port] => (port.parse().expect("failed to parse port"), None),
            [_] => (8080, None),
            _ => {
                stderr_tx
                    .write(format!("invalid arguments: {args:?}").into())
                    .await;
                return Err(());
            }
        };

        let (trailers_tx, trailers_rx) = wit_future::new();
        let (request, transmit) = Request::new(Headers::new(), None, trailers_rx, None);
        let authority = format!("127.0.0.1:{port}");
        request
            .set_scheme(Some(&Scheme::Http))
            .expect("failed to set scheme");
        request
            .set_authority(Some(&authority))
            .expect("failed to set authority");
        if let Some(pq) = pq.as_ref() {
            request
                .set_path_with_query(Some(&pq))
                .expect("failed to set path_with_query");
        }
        let pq = pq.map_or("/", |pq| pq);
        stderr_tx
            .write(format!("sending GET request to {authority}{pq}\n",).into())
            .await;
        let response = match handle(request).await {
            Ok(response) => response,
            Err(err) => {
                stderr_tx
                    .write(format!("failed to send request: {err:?}\n",).into())
                    .await;
                return Err(());
            }
        };
        let status = response.status_code();
        stdout_tx.write(format!("Status: {status}\n").into()).await;
        let headers = response.headers().entries();
        if !headers.is_empty() {
            stdout_tx.write("\nHeaders:\n".into()).await;
            for (k, v) in headers {
                stdout_tx
                    .write(format!("{k}: {}\n", String::from_utf8_lossy(&v)).into())
                    .await;
            }
        }
        join!(
            async {
                transmit
                    .await
                    .expect("transmit sender dropped")
                    .expect("failed to transmit request")
            },
            async {
                let (body_rx, trailers_rx) = response.body().expect("failed to get response body");
                join!(
                    async {
                        // This can fail in HTTP/1.1, since the connection might already be closed
                        _ = trailers_tx.write(Ok(None)).await;
                    },
                    async {
                        let body = body_rx.collect().await;
                        let trailers = trailers_rx
                            .await
                            .expect("trailers sender dropped")
                            .expect("failed to read body");
                        stdout_tx.write("\nBody:\n".into()).await;
                        stdout_tx.write(body).await;
                        stdout_tx.write("\n".into()).await;
                        if let Some(trailers) = trailers {
                            let trailers = trailers.entries();
                            if !trailers.is_empty() {
                                stdout_tx.write("\nTrailers:\n".into()).await;
                                for (k, v) in trailers {
                                    stdout_tx
                                        .write(
                                            format!("{k}: {}\n", String::from_utf8_lossy(&v))
                                                .into(),
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                );
            },
        );
        Ok(())
    }
}
