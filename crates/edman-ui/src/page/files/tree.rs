use std::collections::BTreeMap;

use iced::{
    theme,
    widget::{button, column, horizontal_space, row, text},
    Color, Element, Length, Padding, Theme,
};

use crate::{grpc, icon};

#[derive(Debug, Clone)]
pub struct TreeView {
    tree: BTreeMap<Vec<String>, GraphicsEntry>,
}

#[derive(Debug, Clone)]
pub struct GraphicsEntry {
    depth: usize,
    is_open: bool,
    id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Leaf {
    pub id: i32,
    pub path: Vec<String>,
}

impl From<grpc::ui::File> for Leaf {
    fn from(value: grpc::ui::File) -> Self {
        Leaf {
            id: value.id,
            path: value.path.split("/").map(str::to_string).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeViewMessage {
    path: Vec<String>,
}

impl TreeView {
    pub fn new<T: Into<Leaf>>(leaves: impl Iterator<Item = T>) -> Self {
        let tree = BTreeMap::from_iter(leaves.flat_map(|to_leaf| {
            let leaf: Leaf = to_leaf.into();

            let mut partial_tree: Vec<(Vec<String>, GraphicsEntry)> =
                Vec::with_capacity(leaf.path.len());
            for depth in 0..leaf.path.len() {
                let entry = GraphicsEntry {
                    depth,
                    is_open: false,
                    id: if depth == leaf.path.len() - 1 {
                        //last
                        Some(leaf.id)
                    } else {
                        None
                    },
                };
                partial_tree.push((leaf.path[0..depth + 1].to_vec(), entry));
            }
            partial_tree
        }));

        Self { tree }
    }
    pub fn view(&self) -> Element<TreeViewMessage> {
        let buttons: Vec<Element<TreeViewMessage>> = self
            .tree
            .iter()
            .scan(None, |depth: &mut Option<usize>, (k, v)| {
                if v.is_open {
                    *depth = match depth {
                        Some(depth) if *depth >= k.len() => None,
                        Some(depth) => Some(*depth),
                        None => None,
                    };
                } else {
                    *depth = Some(k.len().min(depth.unwrap_or(usize::MAX)));
                }

                Some(if matches!(depth, Some(l) if *l<k.len()) {
                    None
                } else {
                    Some((k, v))
                })
            })
            .filter_map(|d| d)
            .map(|(k, v)| {
                let file_name = k.last().map(|s| &s[..]).unwrap_or("");
                let indicator = if v.is_open {
                    icon::ANGLE_DOWN
                } else {
                    icon::ANGLE_RIGHT
                };

                button(row![
                    text(indicator).font(icon::ICON_FONT).width(15),
                    horizontal_space(5),
                    file_name
                ])
                .width(Length::Fill)
                .style(theme::Button::Custom(Box::new(CustomButtonStyle)))
                .padding(Padding {
                    top: 0.,
                    right: 0.,
                    bottom: 0.,
                    left: 20. * (v.depth as f32),
                })
                .on_press(TreeViewMessage { path: k.to_owned() })
                .into()
            })
            .collect();

        column(buttons).into()
    }
    pub fn update(&mut self, message: TreeViewMessage) {
        match self.tree.get_mut(&message.path) {
            Some(entry) => entry.is_open ^= true,
            None => eprintln!("Node {} not found!", message.path.join("/")),
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
