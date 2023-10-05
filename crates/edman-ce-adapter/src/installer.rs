use std::{io::Write, path::PathBuf};

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
enum BrowserStrain {
    Chromium,
    Firefox,
}

impl From<BrowserKind> for BrowserStrain {
    fn from(value: BrowserKind) -> Self {
        match value {
            BrowserKind::Firefox => Self::Firefox,
            _ => Self::Chromium,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct AppManifest {
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

pub fn install(
    option: BrowserKind,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = match option.into() {
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
    };

    let manifest_str = serde_json::to_string_pretty(&manifest)?;
    let manifest_path = match option.into() {
        BrowserStrain::Chromium => utils::manifest_path_chromium(),
        BrowserStrain::Firefox => utils::manifest_path_firefox(),
    };
    utils::create_parent_dirs(&manifest_path)?;
    std::fs::File::create(&manifest_path)?.write_all(manifest_str.as_bytes())?;

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            let parent_key = get_registry(&option)?;
            let (key, _) = parent_key.create_subkey(utils::EDMAN_UNIQUE_NAME)?;
            key.set_value("", manifest_path.to_str().as_ref().unwrap())?;
        } else if #[cfg(unix)] {
            let link_path = get_link_path(&option);
            std::os::unix::fs::symlink(manifest_path, link_path)?;
        }
    }

    println!("Edman native messaging manifest was successfully installed.");

    Ok(())
}

pub fn uninstall(option: BrowserKind) -> Result<(), Box<dyn std::error::Error>> {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            let parent_key = get_registry(&option)?;
            parent_key.delete_subkey(utils::EDMAN_UNIQUE_NAME)?;
        } else if #[cfg(unix)] {
            let link_path = get_link_path(&option);
            std::fs::remove_file(link_path)?;
        }
    }

    println!("Edman native messaging manifest was successfully uninstalled.");

    Ok(())
}

#[cfg(unix)]
fn get_link_path(option: &BrowserKind) -> PathBuf {
    cfg_if::cfg_if! {
        if #[cfg(target="macos")] {
            let path = match option {
                BrowserKind::Chrome => "Library/Application Support/Chrome/NativeMessagingHosts",
                BrowserKind::Chromium => "Library/Application Support/Chromium/NativeMessagingHosts",
                BrowserKind::Vivaldi => "Library/Application Support/Vivaldi/NativeMessagingHosts",
                BrowserKind::Firefox => "Library/Application Support/Mozilla/NativeMessagingHosts",
            };
        } else {
            let path = match option {
                BrowserKind::Chrome => ".config/chrome/NativeMessagingHosts",
                BrowserKind::Chromium => ".config/chromium/NativeMessagingHosts",
                BrowserKind::Vivaldi => ".config/vivaldi/NativeMessagingHosts",
                BrowserKind::Firefox => ".mozilla/native-messaging-hosts",
            };
        }
    };

    let user_dir = directories::UserDirs::new()
        .map(|user_dir| user_dir.home_dir().to_owned())
        .expect("Could not retrieve home directory.");

    user_dir
        .join(path)
        .join(format!("{}.json", utils::EDMAN_UNIQUE_NAME))
}

#[cfg(windows)]
fn get_registry(option: &BrowserKind) -> std::io::Result<winreg::RegKey> {
    use winreg::enums::*;
    use winreg::RegKey;
    let path = match option.into() {
        BrowserStrain::Chromium => r"SOFTWARE\Google\Chrome\NativeMessagingHosts",
        BrowserStrain::Firefox => r"SOFTWARE\Mozilla\NativeMessagingHosts",
    };

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(path)?;

    Ok(key)
}
