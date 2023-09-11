use std::fmt::Debug;

use iced::{
    font, theme,
    widget::{column, row, text},
    Command, Element,
};
use tonic::transport::Channel;

#[derive(Default)]
pub struct Loading {
    pub states: LoadingStates,
}

#[derive(Debug)]
pub enum LoadingMessage {
    Font(Result<(), font::Error>),
    Icon(Result<(), font::Error>),
    ApiServer(Result<Channel, tonic::transport::Error>),
    Ui,
}

impl Loading {
    pub fn new() -> (Self, Command<LoadingMessage>) {
        (
            Self::default(),
            Command::batch(vec![
                font::load(include_bytes!("../fonts/BIZUDGothic-Regular.ttf").as_slice())
                    .map(LoadingMessage::Font),
                font::load(include_bytes!("../fonts/Font Awesome 6 Free-Solid-900.otf").as_slice())
                    .map(LoadingMessage::Icon),
                Command::perform(transport::connect(), LoadingMessage::ApiServer),
            ]),
        )
    }

    pub fn update(&mut self, message: LoadingMessage) {
        match message {
            LoadingMessage::Font(r) => self.states.font = Some(r),
            LoadingMessage::Icon(r) => self.states.icon = Some(r),
            LoadingMessage::ApiServer(r) => self.states.api_server = Some(r),
            LoadingMessage::Ui => (),
        }
    }

    pub fn view(&self) -> Element<LoadingMessage> {
        self.states.display().map(|_| LoadingMessage::Ui)
    }
}

macro_rules! all {
    ($f:expr, $x_f:expr, $($x:expr),*) => {
        {
            $f($x_f) $( && $f($x))*
        }
    };
}

macro_rules! any {
    ($f:expr, $x_f:expr, $($x:expr),*) => {
        {
            $f($x_f) $( || $f($x))*
        }
    };
}

macro_rules! map {
    ($f:expr, $p:ident, $($x:ident),*) => {
        {
            vec![$($f("$x",&$p.$x)),*]
        }
    };
}

#[derive(Default)]
pub struct LoadingStates {
    pub font: Option<Result<(), font::Error>>,
    pub icon: Option<Result<(), font::Error>>,
    pub api_server: Option<Result<Channel, tonic::transport::Error>>,
}

impl LoadingStates {
    pub fn is_ok(&self) -> bool {
        all!(Self::s_is_ok, &self.font, &self.icon, &self.api_server)
    }
    pub fn is_err(&self) -> bool {
        any!(Self::s_is_err, &self.font, &self.icon, &self.api_server)
    }
    pub fn display(&self) -> Element<()> {
        column(map!(Self::to_display, self, font, icon, api_server)).into()
    }

    fn s_is_ok<T, E>(opt: &Option<Result<T, E>>) -> bool {
        matches!(opt, Some(Ok(_)))
    }
    fn s_is_err<T, E>(opt: &Option<Result<T, E>>) -> bool {
        matches!(opt, Some(Err(_)))
    }

    fn to_display<T, E: Debug>(
        name: &'static str,
        opt: &Option<Result<T, E>>,
    ) -> Element<'static, ()> {
        const COLOR_LOADING: iced::Color = iced::Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.6,
        };
        const COLOR_SUCCESS: iced::Color = iced::Color {
            r: 0.,
            g: 0.66,
            b: 0.26,
            a: 1.,
        }; //#00a960
        const COLOR_ERROR: iced::Color = iced::Color {
            r: 0.91,
            g: 0.22,
            b: 0.24,
            a: 1.,
        }; // #e8383d

        let status = match opt {
            Some(Ok(_)) => text("Loaded").style(theme::Text::Color(COLOR_SUCCESS)),
            Some(Err(err)) => text(format!("{:?}", err)).style(COLOR_ERROR),
            None => text("Loading").style(COLOR_LOADING),
        };
        row![text(name), text(":"), status].into()
    }
}
