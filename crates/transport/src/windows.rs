use tonic::transport::Channel;

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

pub async fn connect() -> Result<Channel, tonic::transport::Error> {
    let connection = tonic::transport::Endpoint::new(format!("http://{}", utils::sock_path()))?
        .connect()
        .await?;
    Ok(connection)
}

pub async fn disconnect() -> Result<(), std::io::Error> {
    Ok(())
}

pub async fn sock_stream() -> Result<TcpListenerStream, Box<dyn std::error::Error>> {
    let addr = utils::sock_path();

    let tcp = TcpListener::bind(addr).await?;
    let tcp_stream = TcpListenerStream::new(tcp);

    println!("Listening at {}", addr);

    Ok(tcp_stream)
}
