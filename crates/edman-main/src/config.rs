use prisma_client_rust::QueryError;

use crate::PrismaClient;

pub use crate::grpc::config::Config;

// TODO: Rust 1.74
#[tonic::async_trait]
pub trait ConfigurationInterface {
    async fn ensure_db(client: &PrismaClient) -> Result<Box<Self>, QueryError>;
}

// TODO: Rust 1.74
#[tonic::async_trait]
impl ConfigurationInterface for Config {
    async fn ensure_db(client: &PrismaClient) -> Result<Box<Self>, QueryError> {
        let config = db_read(client).await?;
        if let Some(config) = config {
            return Ok(Box::new(config));
        }

        let default_config = default();
        db_write_all(client, default_config.clone()).await?;
        Ok(Box::new(default_config))
    }
}

async fn db_read(client: &PrismaClient) -> Result<Option<Config>, QueryError> {
    let config = client
        .config()
        .find_unique(prisma_codegen::config::UniqueWhereParam::IdEquals(0))
        .exec()
        .await?;

    Ok(config.map(|config| Config {
        download_directory: config.download_directory,
        download_subdirectory: config.download_subdirectory,
        save_file_directory: config.save_file_directory,
        allowed_origins: config
            .allowed_origins
            .split("\n")
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        allowed_extensions: config
            .allowed_extensions
            .split("\n")
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    }))
}

async fn db_write_all(client: &PrismaClient, config: Config) -> Result<(), QueryError> {
    client
        .config()
        .create(
            0,
            config.download_directory,
            config.download_subdirectory,
            config.save_file_directory,
            config.allowed_extensions.join("\n"),
            config.allowed_origins.join("\n"),
            vec![],
        )
        .exec()
        .await?;
    Ok(())
}

fn default() -> Config {
    let current_dir = std::env::current_dir();
    let user_dirs = directories::UserDirs::new();

    Config {
        download_directory: user_dirs
            .as_ref()
            .and_then(|user_dirs| user_dirs.download_dir())
            .and_then(|download_dir| download_dir.to_str())
            .map(|s| s.to_string())
            .unwrap_or("".to_string()),
        download_subdirectory: "edman".to_string(),
        save_file_directory: current_dir
            .ok()
            .as_ref()
            .and_then(|d| d.to_str())
            .map(|d| d.to_string())
            .unwrap_or("".to_string()),
        allowed_extensions: vec![],
        allowed_origins: vec![],
    }
}
