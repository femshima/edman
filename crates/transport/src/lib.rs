use tonic::transport::Channel;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use tokio::net::TcpListener;
        use tokio_stream::wrappers::TcpListenerStream;
    } else if #[cfg(unix)] {
        use tokio::net::{UnixListener, UnixStream};
        use tokio_stream::wrappers::UnixListenerStream;
        use std::{ffi::OsStr, os::unix::prelude::OsStrExt, path::PathBuf};
        use tower::service_fn;
        use tonic::transport::{Endpoint, Uri};
    }
}

#[cfg(unix)]
pub async fn connect() -> Result<Channel, tonic::transport::Error> {
    let uri = Uri::builder()
        .authority("localhost")
        .scheme("file")
        .path_and_query(utils::sock_path().as_os_str().as_bytes())
        .build()
        .expect("Could not parse uri. This is not supposed to happen");

    let channel = Endpoint::from(uri)
        .connect_with_connector(service_fn(|uri: Uri| {
            let path_str = OsStr::from_bytes(uri.path().as_bytes());
            UnixStream::connect(PathBuf::from(path_str))
        }))
        .await?;

    Ok(channel)
}

#[cfg(unix)]
pub async fn disconnect() -> Result<(), std::io::Error> {
    let sock_path = utils::sock_path();
    tokio::fs::remove_file(sock_path).await
}

#[cfg(windows)]
pub async fn connect() -> Result<Channel, tonic::transport::Error> {
    let connection = tonic::transport::Endpoint::new(format!("http://{}", utils::sock_path()))?
        .connect()
        .await?;
    Ok(connection)
}

#[cfg(windows)]
pub async fn disconnect() -> Result<(), std::io::Error> {
    Ok(())
}

#[cfg(unix)]
pub async fn sock_stream() -> Result<UnixListenerStream, Box<dyn std::error::Error>> {
    let sock_path = utils::sock_path();
    utils::create_parent_dirs(&sock_path)?;

    let uds = UnixListener::bind(&sock_path)?;
    let uds_stream = UnixListenerStream::new(uds);

    println!("Listening at {}", sock_path.display());

    Ok(uds_stream)
}

#[cfg(windows)]
pub async fn sock_stream() -> Result<TcpListenerStream, Box<dyn std::error::Error>> {
    let addr = utils::sock_path();

    let tcp = TcpListener::bind(addr).await?;
    let tcp_stream = TcpListenerStream::new(tcp);

    println!("Listening at {}", addr);

    Ok(tcp_stream)
}
