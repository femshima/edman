use std::{io::Write, path::PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::config;

const EDMAN_UNIQUE_NAME: &str = "edman";

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
            name: EDMAN_UNIQUE_NAME,
            description: "Manages files",
            messaging_type: "stdio",
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
            let parent_key = get_registry(&option)?;
            let (key, _) = parent_key.create_subkey(EDMAN_UNIQUE_NAME)?;
            key.set_value("", manifest_path.to_str().as_ref().unwrap())?;
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
            let parent_key = get_registry(&option)?;
            parent_key.delete_subkey(EDMAN_UNIQUE_NAME)?;
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
                InstallOptions::Chrome => "Library/Application Support/Chrome/NativeMessagingHosts",
                InstallOptions::Chromium => "Library/Application Support/Chromium/NativeMessagingHosts",
                InstallOptions::Vivaldi => "Library/Application Support/Vivaldi/NativeMessagingHosts",
                InstallOptions::Firefox => "Library/Application Support/Mozilla/NativeMessagingHosts",
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

    user_dir
        .join(path)
        .join(format!("{}.json", EDMAN_UNIQUE_NAME))
}

#[cfg(windows)]
fn get_registry(option: &InstallOptions) -> std::io::Result<winreg::RegKey> {
    use winreg::enums::*;
    use winreg::RegKey;

    let path = match option {
        InstallOptions::Chrome => r"SOFTWARE\Google\Chrome\NativeMessagingHosts",
        InstallOptions::Chromium => r"SOFTWARE\Google\Chrome\NativeMessagingHosts",
        InstallOptions::Vivaldi => r"SOFTWARE\Vivaldi\NativeMessagingHosts",
        InstallOptions::Firefox => r"SOFTWARE\Mozilla\NativeMessagingHosts",
    };

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(path)?;

    Ok(key)
}
