use std::collections::HashMap;

use files::{FileNode, FileNodeMessage, FileTypeEnum};
use iced::widget::{button, column, row, scrollable, text};
use iced::{
    executor, font, theme, Alignment, Application, Color, Command, Element, Font, Length, Padding,
    Settings, Theme,
};

mod files;

pub fn main() -> iced::Result {
    Counter::run(Settings {
        default_font: Font::with_name("BIZ UDGothic"),
        ..Settings::default()
    })
}

struct Counter {
    value: i32,
    folders: FileNode,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
    FontLoaded(Result<(), font::Error>),
    FileMessage(FileNodeMessage),
}

impl Application for Counter {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                value: 0,
                folders: FileNode {
                    id: 0,
                    name: "Root".to_string(),
                    is_open: true,
                    file_type: FileTypeEnum::Node {
                        children: vec![
                            FileNode {
                                id: 1,
                                name: "文学部 hogehoge スパコンプログラミング".to_string(),
                                is_open: false,
                                file_type: FileTypeEnum::Node {
                                    children: vec![FileNode {
                                        id: 1,
                                        name: "123456.pdf".to_string(),
                                        is_open: false,
                                        file_type: FileTypeEnum::Leaf {
                                            attributes: HashMap::from_iter(
                                                [("path".to_string(), "hoge".to_string())]
                                                    .into_iter(),
                                            ),
                                        },
                                    }],
                                },
                            },
                            FileNode {
                                id: 2,
                                name: "文学部 fugafuga うがー".to_string(),
                                is_open: false,
                                file_type: FileTypeEnum::Node {
                                    children: vec![FileNode {
                                        id: 2,
                                        name: "555555.pdf".to_string(),
                                        is_open: false,
                                        file_type: FileTypeEnum::Leaf {
                                            attributes: HashMap::from_iter(
                                                [("path".to_string(), "hoge".to_string())]
                                                    .into_iter(),
                                            ),
                                        },
                                    }],
                                },
                            },
                        ],
                    },
                },
            },
            font::load(include_bytes!("../fonts/BIZUDGothic-Regular.ttf").as_slice())
                .map(Message::FontLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            Message::FontLoaded(_) => (),
            Message::FileMessage(m) => {
                self.folders.update(m);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let files: Element<_> = self.folders.view().map(|m| Message::FileMessage(m));

        scrollable(
            column![
                button("Increment").on_press(Message::IncrementPressed),
                text(self.value).size(50),
                files,
            ]
            .padding(20)
            .align_items(Alignment::Center),
        )
        .into()
    }
}
