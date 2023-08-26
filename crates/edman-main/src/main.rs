use std::collections::{hash_map::RandomState, HashSet};

use prisma_codegen::PrismaClient;

use ce_adapter::download_manager_server::{DownloadManager, DownloadManagerServer};
use tonic::{transport::Server, Request, Response, Status};

pub mod ce_adapter {
    tonic::include_proto!("chrome_extension");
}
pub mod config {
    tonic::include_proto!("config");
}

fn error_prisma_to_tonic(err: prisma_client_rust::QueryError) -> Status {
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

pub struct ChromeExtensionInterface {
    prisma_client: PrismaClient,
}

// TODO: Rust 1.74
#[tonic::async_trait]
impl DownloadManager for ChromeExtensionInterface {
    async fn get_config(
        &self,
        _request: Request<ce_adapter::ConfigRequest>,
    ) -> Result<Response<ce_adapter::ConfigReply>, Status> {
        let current_dir = std::env::current_dir()
            .map_err(|_err| Status::unavailable("Could not retrieve current dir"))?;
        let user_dirs = directories::UserDirs::new();

        let reply = config::Config {
            download_directory: user_dirs
                .as_ref()
                .and_then(|user_dirs| user_dirs.download_dir())
                .and_then(|download_dir| download_dir.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| todo!()),
            download_subdirectory: "edman".to_string(),
            save_file_directory: current_dir
                .to_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| todo!()),
            allowed_extensions: vec![],
            allowed_origins: vec![],
        };
        Ok(Response::new(ce_adapter::ConfigReply {
            config: Some(reply),
        }))
    }
    async fn get_file_states(
        &self,
        request: Request<ce_adapter::GetFileStatesRequest>,
    ) -> Result<Response<ce_adapter::GetFileStatesReply>, Status> {
        dbg!(request.get_ref());

        let params = request.get_ref();
        let records = self
            .prisma_client
            .file()
            .find_many(vec![prisma_codegen::file::key::in_vec(
                params.keys.to_vec(),
            )])
            .exec()
            .await
            .map_err(error_prisma_to_tonic)?;

        let key_set: HashSet<&str, RandomState> =
            HashSet::from_iter(records.iter().map(|record| &record.key[..]));

        let reply = ce_adapter::GetFileStatesReply {
            result: request
                .get_ref()
                .keys
                .iter()
                .map(|key| key_set.contains(&key[..]))
                .collect(),
        };
        Ok(Response::new(reply))
    }
    async fn register_file(
        &self,
        request: Request<ce_adapter::RegisterFileRequest>,
    ) -> Result<Response<ce_adapter::RegisterFileReply>, Status> {
        dbg!(request.get_ref());

        let params = request.get_ref();
        let record = self
            .prisma_client
            .file()
            .create(params.key.to_owned(), params.path.to_owned(), vec![])
            .exec()
            .await
            .map_err(error_prisma_to_tonic)?;

        let reply = ce_adapter::RegisterFileReply { id: record.id };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prisma_client = PrismaClient::_builder().build().await?;
    prisma_client._migrate_deploy().await?;

    let addr = "[::1]:50051".parse().unwrap();
    let greeter = ChromeExtensionInterface { prisma_client };

    println!("Edman Main Server listening on {}", addr);

    Server::builder()
        .add_service(DownloadManagerServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
