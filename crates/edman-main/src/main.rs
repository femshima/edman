use std::sync::Arc;

use prisma_codegen::PrismaClient;

use tonic::{transport::Server, Status};

mod ce_adapter;
mod config;
mod grpc;
mod ui;

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
    let prisma_client = Arc::new(PrismaClient::_builder().build().await?);
    prisma_client._migrate_deploy().await?;

    let stream = transport::sock_stream().await?;
    let ce_adapter_interface = ce_adapter::ChromeExtensionInterface {
        prisma_client: prisma_client.to_owned(),
    };
    let ui_interface = ui::UiInterface {
        prisma_client: prisma_client.to_owned(),
    };

    let server = Server::builder()
        .add_service(
            grpc::chrome_extension::download_manager_server::DownloadManagerServer::new(
                ce_adapter_interface,
            ),
        )
        .add_service(grpc::ui::edman_main_server::EdmanMainServer::new(
            ui_interface,
        ));

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server_thread = tokio::spawn(async move {
        if let Err(e) = server
            .serve_with_incoming_shutdown(stream, async {
                rx.await.ok();
            })
            .await
        {
            eprintln!("server error: {}", e);
        }
    });

    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    let _ = tx.send(());
    server_thread.await?;

    transport::dispose_socket().await?;

    Ok(())
}
