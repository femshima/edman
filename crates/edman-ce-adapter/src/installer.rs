use std::{io::Write, path::PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Clone, ValueEnum)]
pub enum InstallOptions {
    Chrome,
    Chromium,
    Vivaldi,
    Firefox,
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
            name: "edman",
            description: "Manages files",
            messaging_type: "stdin",
            path: PathBuf::new(),
            allowed_origins: None,
            allowed_extensions: None,
        }
    }
}

pub fn install(
    option: InstallOptions,
    config: &config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = match option {
        InstallOptions::Chrome | InstallOptions::Chromium | InstallOptions::Vivaldi => {
            AppManifest {
                path: std::env::current_exe()?,
                allowed_origins: Some(config.allowed_origins.to_owned()),
                ..Default::default()
            }
        }
        InstallOptions::Firefox => AppManifest {
            path: std::env::current_exe()?,
            allowed_extensions: Some(config.allowed_extensions.to_owned()),
            ..Default::default()
        },
    };

    let manifest_str = serde_json::to_string_pretty(&manifest)?;
    let manifest_path = std::env::current_exe()?
        .parent()
        .expect("Could not generate manifest path")
        .join("manifest.json");
    std::fs::File::create(&manifest_path)?.write_all(manifest_str.as_bytes())?;

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            todo!("Windowsのことなんて知らないっ！");
        } else if #[cfg(unix)] {
            let link_path = get_link_path(&option);
            std::os::unix::fs::symlink(manifest_path, link_path)?;
        }
    }

    println!("Edman native messaging manifest was successfully installed.");

    Ok(())
}

pub fn uninstall(option: InstallOptions) -> Result<(), Box<dyn std::error::Error>> {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            todo!("Windowsのことなんて知らないっ！");
        } else if #[cfg(unix)] {
            let link_path = get_link_path(&option);
            std::fs::remove_file(link_path)?;
        }
    }

    println!("Edman native messaging manifest was successfully uninstalled.");

    Ok(())
}

#[cfg(unix)]
fn get_link_path(option: &InstallOptions) -> PathBuf {
    cfg_if::cfg_if! {
        if #[cfg(target="macos")] {
            let path = match option {
                InstallOptions::Chrome => "/Library/Application Support/Chrome/NativeMessagingHosts",
                InstallOptions::Chromium => "/Library/Application Support/Chromium/NativeMessagingHosts",
                InstallOptions::Vivaldi => "/Library/Application Support/Vivaldi/NativeMessagingHosts",
                InstallOptions::Firefox => "/Library/Application Support/Mozilla/NativeMessagingHosts",
            };
        } else {
            let path = match option {
                InstallOptions::Chrome => ".config/chrome/NativeMessagingHosts",
                InstallOptions::Chromium => ".config/chromium/NativeMessagingHosts",
                InstallOptions::Vivaldi => ".config/vivaldi/NativeMessagingHosts",
                InstallOptions::Firefox => ".mozilla/native-messaging-hosts",
            };
        }
    };

    let user_dir = directories::UserDirs::new()
        .map(|user_dir| user_dir.home_dir().to_owned())
        .expect("Could not retrieve home directory.");

    user_dir.join(path).join("edman.json")
}
