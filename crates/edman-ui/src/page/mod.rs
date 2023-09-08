mod config;
mod files;

use iced::widget::{column, scrollable};
use iced::{Alignment, Command, Element};

// use self::config::{ConfigSettings, ConfigSettingsMessage};
use self::files::{FileView, FileViewMessage};

pub struct Page {
    file_view: FileView,
    // configs: ConfigSettings,
}

#[derive(Debug, Clone)]
pub enum PageMessage {
    FileMessage(FileViewMessage),
    // ConfigMessage(ConfigSettingsMessage),
}

impl Page {
    pub fn new(grpc_channel: transport::GrpcChannel) -> (Self, Command<PageMessage>) {
        let (file_view, file_view_command) = FileView::new(grpc_channel.clone());
        (
            Self { file_view },
            file_view_command.map(PageMessage::FileMessage),
        )
    }

    pub fn update(&mut self, message: PageMessage) -> Command<PageMessage> {
        match message {
            PageMessage::FileMessage(m) => {
                self.file_view.update(m);
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<PageMessage> {
        let files: Element<_> = self.file_view.view().map(PageMessage::FileMessage);

        scrollable(column![files,].padding(20).align_items(Alignment::Center)).into()
    }
}
