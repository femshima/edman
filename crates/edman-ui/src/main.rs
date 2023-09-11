use std::fmt::Debug;

use iced::widget::{column, scrollable, text};
use iced::{Application, Command, Element, Font, Settings};
use loading::{Loading, LoadingMessage};
use page::{Page, PageMessage};

mod icon;
mod grpc;
mod loading;
mod page;

pub fn main() -> iced::Result {
    App::run(Settings {
        default_font: Font::with_name("BIZ UDGothic"),
        ..Settings::default()
    })
}

enum App {
    Loading(Loading),
    InitPage,
    Loaded(Page),
}

#[derive(Debug)]
enum Message {
    LoadingMessage(LoadingMessage),
    PageMessage(PageMessage),
}

impl Application for App {
    type Message = Message;
    type Flags = ();
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let (loading, batch) = Loading::new();
        (Self::Loading(loading), batch.map(Message::LoadingMessage))
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Self::Loading(loading) => match message {
                Message::LoadingMessage(message) => loading.update(message),
                Message::PageMessage(_) => unreachable!(),
            },
            Self::InitPage => unreachable!(),
            Self::Loaded(page) => match message {
                Message::LoadingMessage(_) => unreachable!(),
                Message::PageMessage(message) => {
                    return page.update(message).map(Message::PageMessage);
                }
            },
        }

        if matches!(self,Self::Loading(loading) if loading.states.is_ok()) {
            let Self::Loading(loading) = std::mem::replace(self, Self::InitPage) else {
                unreachable!()
            };
            let channel = loading.states.api_server.unwrap().unwrap();
            let (page, page_command) = Page::new(channel);
            *self = Self::Loaded(page);
            return page_command.map(Message::PageMessage);
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Loading(loading) => {
                let title = if matches!(self,Self::Loading(loading) if loading.states.is_err()) {
                    text("Error!").size(40)
                } else {
                    text("Loading...").size(40)
                };

                let statuses: Element<Message> =
                    scrollable(loading.view().map(Message::LoadingMessage)).into();

                column![title, statuses].into()
            }
            Self::InitPage => unreachable!(),
            Self::Loaded(page) => page.view().map(Message::PageMessage),
        }
    }
}
