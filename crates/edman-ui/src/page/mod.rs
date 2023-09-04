mod config;
mod files;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use files::{FileNode, FileNodeMessage, FileTypeEnum};
use iced::widget::{button, column, scrollable, text};
use iced::{Alignment, Command, Element};
use tonic::transport::Channel;

pub struct Page {
    folders: FileNode,
    grpc_channel: Rc<RefCell<Channel>>,
}

#[derive(Debug, Clone, Copy)]
pub enum PageMessage {
    FileMessage(FileNodeMessage),
}

impl Page {
    pub fn new(grpc_channel: Channel) -> Self {
        Self {
            grpc_channel: Rc::new(RefCell::new(grpc_channel)),
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
                                            [("path".to_string(), "hoge".to_string())].into_iter(),
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
                                            [("path".to_string(), "hoge".to_string())].into_iter(),
                                        ),
                                    },
                                }],
                            },
                        },
                    ],
                },
            },
        }
    }

    pub fn update(&mut self, message: PageMessage) -> Command<PageMessage> {
        match message {
            PageMessage::FileMessage(m) => {
                self.folders.update(m);
            }
        }

        Command::none()
    }

    pub fn view(&self) -> Element<PageMessage> {
        let files: Element<_> = self.folders.view().map(PageMessage::FileMessage);

        scrollable(
            column![
                files,
            ]
            .padding(20)
            .align_items(Alignment::Center),
        )
        .into()
    }
}
