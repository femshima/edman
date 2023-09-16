use http::Uri;
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};
use tonic::transport::{Channel, Endpoint};
use tower::service_fn;

use self::named_pipe_stream::NamedPipeStream;

pub async fn connect() -> Result<Channel, tonic::transport::Error> {
    let uri = Uri::builder()
        .authority("localhost")
        .scheme("file")
        .path_and_query(utils::sock_path().as_bytes())
        .build()
        .expect("Could not parse uri. This is not supposed to happen");

    let channel = Endpoint::from(uri)
        .connect_with_connector(service_fn(|uri: Uri| {
            let path_str = uri.path_and_query().unwrap().as_str().to_owned();
            async move { ClientOptions::new().open(path_str) }
        }))
        .await?;

    Ok(channel)
}

pub async fn sock_stream() -> Result<NamedPipeStream, Box<dyn std::error::Error>> {
    let addr = utils::sock_path();
    let mut options = ServerOptions::new();
    options.max_instances(8);
    let stream = NamedPipeStream::new(options, addr.to_string());

    println!("Listening at {}", addr);

    Ok(stream)
}

pub async fn dispose_socket() -> Result<(), std::io::Error> {
    Ok(())
}

pub mod named_pipe_stream {
    use std::{
        future::Future,
        ops::{Deref, DerefMut},
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    };

    use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
    use tokio_retry::{strategy::ExponentialBackoff, RetryIf};
    use tokio_stream::Stream;
    use tonic::transport::server::Connected;

    type ConnectionFuture = Pin<Box<dyn Send + Future<Output = std::io::Result<NamedPipeServer>>>>;

    pub struct NamedPipeStream {
        options: ServerOptions,
        addr: String,
        current: ConnectionFuture,
    }

    impl NamedPipeStream {
        pub fn new(options: ServerOptions, addr: String) -> Self {
            let mut this = Self {
                options,
                addr,
                current: Box::pin(std::future::pending()),
            };
            this.renew_connection(true);
            this
        }
        fn renew_connection(&mut self, first: bool) {
            let mut options = self.options.clone();
            options.first_pipe_instance(first);

            let addr = self.addr.to_string();

            let conn: ConnectionFuture = Box::pin(Self::connect(options, addr));
            let _ = std::mem::replace(&mut self.current, conn);
        }
        async fn connect(options: ServerOptions, addr: String) -> std::io::Result<NamedPipeServer> {
            let server = RetryIf::spawn(
                ExponentialBackoff::from_millis(50).max_delay(Duration::new(10, 0)),
                move || std::future::ready(options.create(&addr)),
                |e: &std::io::Error| {
                    // ERROR_PIPE_BUSY: 231
                    // Retry only if the cause is "ERROR_PIPE_BUSY"
                    matches!(e.raw_os_error(), Some(231))
                },
            )
            .await?;

            server.connect().await?;
            Ok(server)
        }
    }

    impl Stream for NamedPipeStream {
        type Item = std::io::Result<Pin<WrappedNamedPipe>>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.current.as_mut().poll(cx) {
                Poll::Ready(Ok(server)) => {
                    self.renew_connection(false);
                    let wrapped_server = WrappedNamedPipe::new(server);
                    Poll::Ready(Some(Ok(Pin::new(wrapped_server))))
                }
                Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    // This wrapper is needed for `Connected` implementation for the server
    #[derive(Debug)]
    pub struct WrappedNamedPipe {
        inner: NamedPipeServer,
    }
    impl WrappedNamedPipe {
        pub fn new(inner: NamedPipeServer) -> Self {
            Self { inner }
        }
    }
    impl Deref for WrappedNamedPipe {
        type Target = NamedPipeServer;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl DerefMut for WrappedNamedPipe {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl Connected for Pin<WrappedNamedPipe> {
        type ConnectInfo = ();
        fn connect_info(&self) -> Self::ConnectInfo {}
    }
}
