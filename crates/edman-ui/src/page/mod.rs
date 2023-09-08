mod config;
mod files;

use iced::widget::{column, row, scrollable};
use iced::{Alignment, Command, Element};

use self::config::{ConfigSettings, ConfigSettingsMessage};
use self::files::{FileView, FileViewMessage};

pub struct Page {
    file_view: FileView,
    configs: ConfigSettings,
}

#[derive(Debug, Clone)]
pub enum PageMessage {
    FileMessage(FileViewMessage),
    ConfigMessage(ConfigSettingsMessage),
}

impl Page {
    pub fn new(grpc_channel: transport::GrpcChannel) -> (Self, Command<PageMessage>) {
        let (file_view, file_view_command) = FileView::new(grpc_channel.clone());
        let (configs, config_command) = ConfigSettings::new(grpc_channel.clone());
        (
            Self { file_view, configs },
            Command::batch(vec![
                file_view_command.map(PageMessage::FileMessage),
                config_command.map(PageMessage::ConfigMessage),
            ]),
        )
    }

    pub fn update(&mut self, message: PageMessage) -> Command<PageMessage> {
        match message {
            PageMessage::FileMessage(m) => {
                self.file_view.update(m);
            }
            PageMessage::ConfigMessage(m) => {
                return self.configs.update(m).map(PageMessage::ConfigMessage);
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<PageMessage> {
        let files: Element<_> = self.file_view.view().map(PageMessage::FileMessage);

        row![
            scrollable(column![files,].padding(20).align_items(Alignment::Center)),
            self.configs.view().map(PageMessage::ConfigMessage)
        ]
        .into()
    }
}
