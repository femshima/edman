use std::{
    cell::RefCell,
    ffi::OsStr,
    path::PathBuf,
    rc::Rc,
    task::{Context, Poll},
};

use tonic::{
    body::BoxBody,
    transport::{Channel, Endpoint, Uri},
};
use tower::{service_fn, Service};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use uds_windows::{UnixListener, UnixStream};
    } else if #[cfg(unix)] {
        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use std::os::unix::prelude::OsStrExt;
    }
}

#[cfg(unix)]
pub async fn connect() -> Result<Channel, Box<dyn std::error::Error>> {
    let uri = Uri::builder()
        .authority("localhost")
        .scheme("file")
        .path_and_query(utils::sock_path().as_os_str().as_bytes())
        .build()?;

    let channel = Endpoint::try_from(uri)?
        .connect_with_connector(service_fn(|uri: Uri| {
            let path_str = OsStr::from_bytes(uri.path().as_bytes());
            UnixStream::connect(PathBuf::from(path_str))
        }))
        .await?;

    Ok(channel)
}

#[cfg(unix)]
pub fn sock_stream() -> Result<UnixListenerStream, Box<dyn std::error::Error>> {
    let sock_path = utils::sock_path();
    utils::create_parent_dirs(&sock_path)?;

    let uds = UnixListener::bind(&sock_path)?;
    let uds_stream = UnixListenerStream::new(uds);

    println!("Listening at {}", sock_path.display());

    Ok(uds_stream)
}

pub struct GrpcChannel {
    inner: Rc<RefCell<Channel>>,
}

impl GrpcChannel {
    pub fn new(inner: Rc<RefCell<Channel>>) -> Self {
        Self { inner }
    }
}

impl Service<http::Request<BoxBody>> for GrpcChannel {
    type Response = http::Response<tonic::transport::Body>;
    type Error = tonic::transport::Error;
    type Future = tonic::transport::channel::ResponseFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Ok(mut channel) = self.inner.try_borrow_mut() {
            channel.poll_ready(cx)
        } else {
            Poll::Pending
        }
    }

    fn call(&mut self, request: http::Request<BoxBody>) -> Self::Future {
        self.inner.borrow_mut().call(request)
    }
}
