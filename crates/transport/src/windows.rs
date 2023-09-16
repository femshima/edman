use std::{
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

use http::Uri;
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions};
use tokio_stream::Stream;
use tonic::transport::{server::Connected, Channel, Endpoint};
use tower::service_fn;

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

pub struct NamedPipeStream {
    options: ServerOptions,
    addr: String,
    current: Pin<Box<dyn Send + Future<Output = std::io::Result<NamedPipeServer>>>>,
}

impl NamedPipeStream {
    pub fn new(options: ServerOptions, addr: String) -> std::io::Result<Self> {
        let mut this = Self {
            options,
            addr,
            current: Box::pin(std::future::pending()),
        };
        this.renew_connection(true)?;
        Ok(this)
    }
    fn renew_connection(&mut self, first: bool) -> std::io::Result<()> {
        let mut options = self.options.clone();
        options.first_pipe_instance(first);
        let next_server = options.create(&self.addr)?;

        let server = Box::pin(Self::connect(next_server));
        let _ = std::mem::replace(&mut self.current, server);

        Ok(())
    }
    async fn connect(server: NamedPipeServer) -> std::io::Result<NamedPipeServer> {
        server.connect().await?;
        Ok(server)
    }
}

impl Stream for NamedPipeStream {
    type Item = std::io::Result<Pin<WrappedNamedPipe>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.current.as_mut().poll(cx) {
            Poll::Ready(Ok(server)) => {
                self.renew_connection(false).unwrap();
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

pub async fn sock_stream() -> Result<NamedPipeStream, Box<dyn std::error::Error>> {
    let addr = utils::sock_path();
    let mut options = ServerOptions::new();
    options.max_instances(8);
    let stream = NamedPipeStream::new(options, addr.to_string())?;

    println!("Listening at {}", addr);

    Ok(stream)
}

pub async fn dispose_socket() -> Result<(), std::io::Error> {
    Ok(())
}
