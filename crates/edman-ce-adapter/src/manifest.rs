use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Clone, Copy, ValueEnum)]
pub enum BrowserKind {
    Chrome,
    Chromium,
    Vivaldi,
    Firefox,
}

#[derive(Clone, Copy)]
pub enum BrowserStrain {
    Chromium,
    Firefox,
}

impl From<&BrowserKind> for BrowserStrain {
    fn from(value: &BrowserKind) -> Self {
        match value {
            BrowserKind::Firefox => Self::Firefox,
            _ => Self::Chromium,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AppManifest {
    name: &'static str,
    description: &'static str,
    path: PathBuf,
    #[serde(rename = "type")]
    messaging_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_origins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_extensions: Option<Vec<String>>,
}

impl Default for AppManifest {
    fn default() -> Self {
        Self {
            name: utils::EDMAN_UNIQUE_NAME,
            description: "Manages files",
            messaging_type: "stdio",
            path: PathBuf::new(),
            allowed_origins: None,
            allowed_extensions: None,
        }
    }
}

impl AppManifest {
    pub fn new(option: &BrowserKind, config: &config::Config) -> std::io::Result<Self> {
        Ok(match option.into() {
            BrowserStrain::Chromium => AppManifest {
                path: std::env::current_exe()?,
                allowed_origins: Some(config.allowed_origins.to_owned()),
                ..Default::default()
            },
            BrowserStrain::Firefox => AppManifest {
                path: std::env::current_exe()?,
                allowed_extensions: Some(config.allowed_extensions.to_owned()),
                ..Default::default()
            },
        })
    }
}
