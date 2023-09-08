use iced::{
    widget::{button, column, row, text, text_input},
    Command, Element,
};
use tonic::Request;

use crate::grpc;

pub struct ConfigSettings {
    channel: tonic::transport::Channel,

    config: Option<grpc::config::Config>,
    allowed_origin: String,
    allowed_extension: String,
}

#[derive(Debug, Clone)]
pub enum ConfigSettingsMessage {
    Loaded(Result<Option<grpc::config::Config>, tonic::Status>),

    StartUpdate,

    ConfigUpdate(ConfigUpdateMessage),

    ChromiumOriginChange(String),
    FirefoxExtensionChange(String),
}

#[derive(Debug, Clone)]
pub enum ConfigUpdateMessage {
    DownloadDirectoryChange(String),
    DownloadSubDirectoryChange(String),
    SaveFileDirectoryChange(String),

    ChromiumOriginAdd,
    FirefoxExtensionAdd,
    ChromiumOriginRemove(usize),
    FirefoxExtensionRemove(usize),
}

impl ConfigSettings {
    pub fn new(channel: tonic::transport::Channel) -> (Self, Command<ConfigSettingsMessage>) {
        (
            Self {
                channel: channel.clone(),
                config: None,
                allowed_origin: String::new(),
                allowed_extension: String::new(),
            },
            Command::perform(
                Self::fetch_config(channel.clone()),
                ConfigSettingsMessage::Loaded,
            ),
        )
    }

    pub fn view(&self) -> Element<ConfigSettingsMessage> {
        let Some(ref config)= self.config else {
            return text("loading").into()
        };

        let directory_settings: Element<ConfigUpdateMessage> = column![
            "Download",
            row![
                text_input("base dir", &config.download_directory[..])
                    .on_input(ConfigUpdateMessage::DownloadDirectoryChange),
                "/",
                text_input("sub dir", &config.download_subdirectory[..])
                    .on_input(ConfigUpdateMessage::DownloadSubDirectoryChange)
            ],
            "Save",
            text_input("save dir", &config.save_file_directory[..])
                .on_input(ConfigUpdateMessage::SaveFileDirectoryChange),
        ]
        .into();

        column![
            directory_settings.map(ConfigSettingsMessage::ConfigUpdate),
            "Ext[Cr]",
            row![
                text_input("Chromium origin", &self.allowed_origin[..])
                    .on_input(ConfigSettingsMessage::ChromiumOriginChange),
                button("+").on_press(ConfigSettingsMessage::ConfigUpdate(
                    ConfigUpdateMessage::ChromiumOriginAdd
                )),
            ],
            column(
                config
                    .allowed_origins
                    .iter()
                    .enumerate()
                    .map(|(i, s)| row![
                        text(s),
                        button("x").on_press(ConfigSettingsMessage::ConfigUpdate(
                            ConfigUpdateMessage::ChromiumOriginRemove(i)
                        ))
                    ]
                    .into())
                    .collect()
            ),
            "Ext[FF]",
            row![
                text_input("Firefox extensions", &self.allowed_extension[..])
                    .on_input(ConfigSettingsMessage::FirefoxExtensionChange),
                button("+").on_press(ConfigSettingsMessage::ConfigUpdate(
                    ConfigUpdateMessage::FirefoxExtensionAdd
                )),
            ],
            column(
                config
                    .allowed_extensions
                    .iter()
                    .enumerate()
                    .map(|(i, s)| row![
                        text(s),
                        button("x").on_press(ConfigSettingsMessage::ConfigUpdate(
                            ConfigUpdateMessage::FirefoxExtensionRemove(i)
                        ))
                    ]
                    .into())
                    .collect()
            ),
            button("Update").on_press(ConfigSettingsMessage::StartUpdate)
        ]
        .into()
    }

    pub fn update(&mut self, message: ConfigSettingsMessage) -> Command<ConfigSettingsMessage> {
        match message {
            ConfigSettingsMessage::Loaded(result) => match result {
                Ok(config) => self.config = config,
                Err(err) => eprintln!("{}", err),
            },

            ConfigSettingsMessage::ChromiumOriginChange(s) => self.allowed_origin = s,
            ConfigSettingsMessage::FirefoxExtensionChange(s) => self.allowed_extension = s,

            ConfigSettingsMessage::StartUpdate => {
                return Command::perform(
                    Self::update_config(self.channel.clone(), self.config.to_owned()),
                    ConfigSettingsMessage::Loaded,
                )
            }

            ConfigSettingsMessage::ConfigUpdate(cfg_update) => {
                let config = self.config.as_mut().expect("Load failed");

                match cfg_update {
                    ConfigUpdateMessage::DownloadDirectoryChange(s) => {
                        config.download_directory = s;
                    }
                    ConfigUpdateMessage::DownloadSubDirectoryChange(s) => {
                        config.download_subdirectory = s;
                    }
                    ConfigUpdateMessage::SaveFileDirectoryChange(s) => {
                        config.save_file_directory = s;
                    }

                    ConfigUpdateMessage::ChromiumOriginAdd => {
                        config.allowed_origins.push(self.allowed_origin.to_owned());
                    }
                    ConfigUpdateMessage::FirefoxExtensionAdd => {
                        config
                            .allowed_extensions
                            .push(self.allowed_extension.to_owned());
                    }

                    ConfigUpdateMessage::ChromiumOriginRemove(i) => {
                        config.allowed_origins.remove(i);
                    }
                    ConfigUpdateMessage::FirefoxExtensionRemove(i) => {
                        config.allowed_origins.remove(i);
                    }
                };
            }
        }

        Command::none()
    }

    async fn fetch_config(
        channel: tonic::transport::Channel,
    ) -> Result<Option<grpc::config::Config>, tonic::Status> {
        let mut client = grpc::Client::new(channel.clone());
        let response = client
            .get_config(Request::new(grpc::ui::ConfigRequest {}))
            .await?;
        Ok(response.get_ref().config.to_owned())
    }

    async fn update_config(
        channel: tonic::transport::Channel,
        config: Option<grpc::config::Config>,
    ) -> Result<Option<grpc::config::Config>, tonic::Status> {
        let mut client = grpc::Client::new(channel.clone());
        let response = client
            .set_config(Request::new(grpc::ui::UpdateConfigRequest { config }))
            .await?;
        Ok(response.get_ref().config.to_owned())
    }
}
