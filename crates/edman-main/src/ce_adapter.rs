use std::collections::{hash_map::RandomState, HashSet};

use prisma_codegen::PrismaClient;

use tonic::{Request, Response, Status};

pub use chrome_extension::download_manager_server::{DownloadManager, DownloadManagerServer};

use crate::config::ConfigurationInterface;
use crate::error_prisma_to_tonic;

pub mod chrome_extension {
    #![allow(non_snake_case)]
    tonic::include_proto!("chrome_extension");
}

pub struct ChromeExtensionInterface {
    prisma_client: PrismaClient,
}

// TODO: Rust 1.74
#[tonic::async_trait]
impl DownloadManager for ChromeExtensionInterface {
    async fn get_config(
        &self,
        _request: Request<chrome_extension::ConfigRequest>,
    ) -> Result<Response<chrome_extension::ConfigReply>, Status> {
        let config = crate::config::Config::ensure_db(&self.prisma_client)
            .await
            .map_err(error_prisma_to_tonic)?;

        Ok(Response::new(chrome_extension::ConfigReply {
            config: Some(*config),
        }))
    }
    async fn get_file_states(
        &self,
        request: Request<chrome_extension::GetFileStatesRequest>,
    ) -> Result<Response<chrome_extension::GetFileStatesReply>, Status> {
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

        let reply = chrome_extension::GetFileStatesReply {
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
        request: Request<chrome_extension::RegisterFileRequest>,
    ) -> Result<Response<chrome_extension::RegisterFileReply>, Status> {
        dbg!(request.get_ref());

        let params = request.get_ref();
        let record = self
            .prisma_client
            .file()
            .create(params.key.to_owned(), params.path.to_owned(), vec![])
            .exec()
            .await
            .map_err(error_prisma_to_tonic)?;

        let reply = chrome_extension::RegisterFileReply { id: record.id };
        Ok(Response::new(reply))
    }
}
