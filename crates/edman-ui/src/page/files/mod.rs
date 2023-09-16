use iced::{widget::text, Command, Element};
use tonic::Request;

mod tree;

use crate::grpc;

use self::tree::{TreeView, TreeViewMessage};

pub struct FileView {
    tree: Option<TreeView>,
}

#[derive(Debug, Clone)]
pub enum FileViewMessage {
    Loaded(Result<Vec<grpc::ui::File>, tonic::Status>),
    TreeMessage(TreeViewMessage),
}

impl FileView {
    pub fn new(channel: tonic::transport::Channel) -> (Self, Command<FileViewMessage>) {
        let client = grpc::Client::new(channel);

        (
            Self { tree: None },
            Command::perform(Self::fetch_files(client), FileViewMessage::Loaded),
        )
    }

    pub fn view(&self) -> Element<FileViewMessage> {
        match self.tree {
            Some(ref tree) => tree.view().map(FileViewMessage::TreeMessage),
            None => text("loading").into(),
        }
    }

    pub fn update(&mut self, message: FileViewMessage) {
        match message {
            FileViewMessage::Loaded(result) => match result {
                Ok(leaves) => self.tree = Some(TreeView::new(leaves.into_iter())),
                Err(err) => eprintln!("{}", err),
            },
            FileViewMessage::TreeMessage(message) => match self.tree {
                Some(ref mut tree) => tree.update(message),
                None => eprintln!("Unknown message in files"),
            },
        }
    }

    async fn fetch_files(mut client: grpc::Client) -> Result<Vec<grpc::ui::File>, tonic::Status> {
        let response = client
            .get_files(Request::new(grpc::ui::FilesRequest {}))
            .await?;
        Ok(response.get_ref().files.to_vec())
    }
}
