use std::collections::HashMap;

use iced::{
    theme,
    widget::{button, column, row},
    Color, Element, Length, Padding, Theme,
};

#[derive(Debug, Clone)]
pub struct FileNodeSource {
    pub id: i32,
    pub path: Vec<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub id: i32,
    pub name: String,
    pub is_open: bool,
    pub file_type: FileTypeEnum,
}

#[derive(Debug, Clone)]
pub enum FileTypeEnum {
    Node { children: Vec<FileNode> },
    Leaf { attributes: HashMap<String, String> },
}

#[derive(Debug, Clone, Copy)]
pub struct FileNodeMessage {
    id: i32,
    depth: usize,
}

impl FileNode {
    pub fn view(&self) -> Element<FileNodeMessage> {
        self.view_inner(0)
    }
    fn view_inner(&self, depth: usize) -> Element<FileNodeMessage> {
        let button = button(row!["▼", &self.name[..]])
            .width(Length::Fill)
            .style(theme::Button::Custom(Box::new(CustomButtonStyle)))
            .on_press(FileNodeMessage { id: self.id, depth })
            .into();

        if !self.is_open {
            return button;
        }

        match &self.file_type {
            FileTypeEnum::Node { children } => column![
                button,
                column(
                    children
                        .iter()
                        .map(|f| f.view_inner(depth + 1))
                        .collect::<Vec<_>>(),
                )
                .padding(Padding {
                    top: 0.,
                    right: 0.,
                    bottom: 0.,
                    left: 20.,
                })
            ]
            .spacing(2)
            .into(),
            FileTypeEnum::Leaf { attributes } => button,
        }
    }
    pub fn update(&mut self, message: FileNodeMessage) {
        self.update_inner(message);
    }
    fn update_inner(&mut self, message: FileNodeMessage) -> bool {
        if message.depth == 0 {
            if self.id == message.id {
                self.is_open ^= true;
                true
            } else {
                false
            }
        } else {
            match &mut self.file_type {
                FileTypeEnum::Node { children } => children.iter_mut().fold(false, |acc, f| {
                    if !acc {
                        f.update_inner(FileNodeMessage {
                            id: message.id,
                            depth: message.depth - 1,
                        })
                    } else {
                        true
                    }
                }),
                FileTypeEnum::Leaf { .. } => false,
            }
        }
    }
}

struct CustomButtonStyle;

impl button::StyleSheet for CustomButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(Color::WHITE)),
            border_width: 0.0,
            ..Default::default()
        }
    }
}
