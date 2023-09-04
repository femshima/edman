use std::fmt::Display;

use iced::widget::{column, scrollable, text};
use iced::{font, Application, Command, Element, Font, Settings};
use page::{Page, PageMessage};
use strum::EnumCount;
use strum_macros::{EnumCount, IntoStaticStr};
use tonic::transport::Channel;

mod grpc;
mod page;

pub fn main() -> iced::Result {
    App::run(Settings {
        default_font: Font::with_name("BIZ UDGothic"),
        ..Settings::default()
    })
}

#[derive(Default)]
struct App {
    status: Status,
    loading_state: Option<LoadingStates>,
}

#[derive(Default)]
enum Status {
    #[default]
    Loading,
    Loaded(Page),
    Error,
}

#[derive(Debug)]
enum Message {
    Loaded(LoadMessage),
    PageMessage(PageMessage),
}

#[derive(Debug, EnumCount, IntoStaticStr)]
enum LoadMessage {
    Font(Result<(), font::Error>),
    ApiServer(Result<Channel, tonic::transport::Error>),
}

impl LoadMessage {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Font(inner) => inner.is_ok(),
            Self::ApiServer(inner) => inner.is_ok(),
        }
    }
    pub fn is_err(&self)->bool{
        !self.is_ok()
    }
}

struct LoadingStates{
    font: Option<Result<(),font::Error>>,
    api_server: Option<Result<Channel, tonic::transport::Error>>
}

impl LoadingStates{
    pub fn is_status_determined(&self)->bool{
        macro_rules! any_is_err {
            ($f:item,$($x:expr),*) => (
                $()
            );
        }

        let err=Self::is_err(&self.font)|Self::is_err(&self.api_server);
        let ok=
    }
    fn is_ok<T,E>(opt: &Option<Result<T,E>>)->bool{
        matches!(opt, Some(Ok(_)))
    }
    fn is_err<T,E>(opt: &Option<Result<T,E>>)->bool{
        matches!(opt, Some(Err(_)))
    }
}

impl Application for App {
    type Message = Message;
    type Flags = ();
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self::default(),
            Command::batch(vec![
                font::load(include_bytes!("../fonts/BIZUDGothic-Regular.ttf").as_slice())
                    .map(LoadMessage::Font),
                Command::perform(transport::connect(), LoadMessage::ApiServer),
            ])
            .map(Message::Loaded),
        )
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(message) => self.loading_state.get_or_insert(vec![]).push(message),
            Message::PageMessage(message) => {
                if let Status::Loaded(ref mut page) = self.status {
                    return page.update(message).map(Message::PageMessage);
                }
            }
        }

        if matches!(self.status, Status::Loading) && matches!(&self.loading_state,Some(v) if v.len() == LoadMessage::COUNT)
        {
            let loading_state = self.loading_state.take().unwrap();

            match loading_state{
                LoadingStates{
                    font: Some(Ok(_)),
                    api_server: Some(Ok(grpc_channel))
                }=>{
                    self.status=Status::Loaded(Page::new(grpc_channel))
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        if let Status::Loaded(ref page) = self.status {
            return page.view().map(Message::PageMessage);
        }

        let title = match self.status {
            Status::Loading { .. } => text("Loading...").size(40),
            Status::Error => text("Error!").size(40),
            _ => unreachable!(),
        };

        let statuses=

        column![title, scrollable(column(self.))].into()
    }
}
