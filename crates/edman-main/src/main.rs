use ce_adapter::DownloadManagerServer;
use prisma_codegen::PrismaClient;

use tonic::{transport::Server, Status};

mod ce_adapter;
mod config;

pub fn error_prisma_to_tonic(err: prisma_client_rust::QueryError) -> Status {
    use prisma_client_rust::prisma_errors::query_engine::*;
    if err.is_prisma_error::<PoolTimeout>() {
        Status::unavailable("Database timeout")
    } else if err.is_prisma_error::<UniqueKeyViolation>() {
        Status::already_exists("Unique key violation")
    } else if err.is_prisma_error::<RecordNotFound>() {
        Status::not_found("Record not found")
    } else if err.is_prisma_error::<ForeignKeyViolation>() {
        Status::failed_precondition("Foreign key violation")
    } else {
        Status::internal(format!("Database error: {:?}", err))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prisma_client = PrismaClient::_builder().build().await?;
    prisma_client._migrate_deploy().await?;

    let stream = transport::sock_stream().await?;
    let greeter = ce_adapter::ChromeExtensionInterface { prisma_client };

    Server::builder()
        .add_service(DownloadManagerServer::new(greeter))
        .serve_with_incoming(stream)
        .await?;

    Ok(())
}
