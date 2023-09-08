use std::sync::Arc;

use prisma_codegen::PrismaClient;

use crate::error_prisma_to_tonic;
use crate::{
    config::ConfigurationInterface,
    grpc::ui::{self, edman_main_server::EdmanMain},
};

use tonic::{Request, Response, Status};

pub struct UiInterface {
    pub prisma_client: Arc<PrismaClient>,
}

// TODO: Rust 1.74
#[tonic::async_trait]
impl EdmanMain for UiInterface {
    async fn get_config(
        &self,
        _request: Request<ui::ConfigRequest>,
    ) -> Result<Response<ui::ConfigReply>, Status> {
        let config = crate::config::Config::ensure_db(&self.prisma_client)
            .await
            .map_err(error_prisma_to_tonic)?;

        Ok(Response::new(ui::ConfigReply {
            config: Some(*config),
        }))
    }

    async fn get_files(
        &self,
        _request: Request<ui::FilesRequest>,
    ) -> Result<Response<ui::FilesReply>, Status> {
        let files = self
            .prisma_client
            .file()
            .find_many(vec![])
            .exec()
            .await
            .map_err(error_prisma_to_tonic)?;

        Ok(Response::new(ui::FilesReply {
            files: files
                .iter()
                .map(|file| ui::File {
                    id: file.id,
                    created_at: file.created_at.timestamp(),
                    key: file.key.to_owned(),
                    path: file.path.to_owned(),
                })
                .collect(),
        }))
    }
}
