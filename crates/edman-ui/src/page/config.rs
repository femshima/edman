use iced::{widget::column, Command, Element};
use tonic::{transport::Channel, Request, Response};

use crate::grpc;

pub struct ConfigSettings {}

pub enum ConfigSettingsMessage {
    Loaded(Result<Response<grpc::ui::ConfigReply>, tonic::Status>),
}

impl ConfigSettings {
    // pub fn new(client: &mut grpc::Client) -> (Self, Command<ConfigSettingsMessage>) {
    //     Self {}
    // }

    pub fn view(&self) -> Element<ConfigSettingsMessage> {
        column![].into()
    }

    pub fn update(&mut self, message: ConfigSettingsMessage) {}
}
