use iced::{
    widget::{column, row, text, text_input},
    Command, Element,
};
use tonic::Request;

use crate::grpc;

pub struct ConfigSettings {
    config: Option<grpc::config::Config>,
}

#[derive(Debug, Clone)]
pub enum ConfigSettingsMessage {
    Loaded(Result<Option<grpc::config::Config>, tonic::Status>),
}

impl ConfigSettings {
    pub fn new(channel: transport::GrpcChannel) -> (Self, Command<ConfigSettingsMessage>) {
        let client = grpc::Client::new(channel.clone());

        (
            Self { config: None },
            Command::perform(Self::fetch_files(client), ConfigSettingsMessage::Loaded),
        )
    }

    pub fn view(&self) -> Element<ConfigSettingsMessage> {
        let Some(ref config)= self.config else {
            return text("loading").into()
        };

        column![
            "Download",
            row![
                text_input("base dir", &config.download_directory[..]),
                "/",
                text_input("sub dir", &config.download_subdirectory[..])
            ],
            "Save",
            text_input("save dir", &config.save_file_directory[..]),
            "Ext[Cr]",
            column(
                config
                    .allowed_origins
                    .iter()
                    .map(|s| text(s).into())
                    .collect()
            ),
            "Ext[FF]",
            column(
                config
                    .allowed_extensions
                    .iter()
                    .map(|s| text(s).into())
                    .collect()
            )
        ]
        .into()
    }

    pub fn update(&mut self, message: ConfigSettingsMessage) {
        match message {
            ConfigSettingsMessage::Loaded(result) => match result {
                Ok(config) => self.config = config,
                Err(err) => eprintln!("{}", err),
            },
        }
    }

    async fn fetch_files(
        mut client: grpc::Client,
    ) -> Result<Option<grpc::config::Config>, tonic::Status> {
        let response = client
            .get_config(Request::new(grpc::ui::ConfigRequest {}))
            .await?;
        Ok(response.get_ref().config.to_owned())
    }
}
