use std::collections::{hash_map::RandomState, HashSet};

use config::ConfigurationInterface;
use prisma_codegen::PrismaClient;

use ce_adapter::download_manager_server::{DownloadManager, DownloadManagerServer};
use tonic::{transport::Server, Request, Response, Status};

mod config;

pub mod ce_adapter {
    #![allow(non_snake_case)]
    tonic::include_proto!("chrome_extension");
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
        let config = config::Config::ensure_db(&self.prisma_client)
            .await
            .map_err(error_prisma_to_tonic)?;

        Ok(Response::new(ce_adapter::ConfigReply {
            config: Some(*config),
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

    let uds_stream = transport::sock_stream()?;
    let greeter = ChromeExtensionInterface { prisma_client };

    Server::builder()
        .add_service(DownloadManagerServer::new(greeter))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}
