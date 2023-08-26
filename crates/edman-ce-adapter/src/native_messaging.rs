use std::{
    io::{Read, Write},
    path::PathBuf,
};

use crate::chrome_extension;
use crate::chrome_extension::download_manager_client::DownloadManagerClient;
use crate::config;
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

#[typeshare::typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum NativeMessage {
    #[serde(rename_all = "camelCase")]
    RegisterFile {
        download_path: String,
        save_path: Vec<String>,
        key: String,
    },
    #[serde(rename_all = "camelCase")]
    FetchFileStates { query: Vec<String> },
}

#[typeshare::typeshare]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum NativeResult {
    #[serde(rename_all = "camelCase")]
    RegisterFile(chrome_extension::RegisterFileReply),
    #[serde(rename_all = "camelCase")]
    FetchFileStates(chrome_extension::GetFileStatesReply),
    Err(String),
}

pub async fn main_loop(
    client: &mut DownloadManagerClient<tonic::transport::Channel>,
    config: &config::Config,
    mut stdin: impl Read,
    mut stdout: impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Ok(size) = stdin.read_u32::<NativeEndian>() {
        let mut input_buf = vec![0u8; size as usize];
        stdin.read_exact(&mut input_buf)?;
        let input_str = String::from_utf8(input_buf)?;

        let native_result = get_reply(client, config, &input_str)
            .await
            .unwrap_or_else(|err| NativeResult::Err(err.to_string()));
        let output_str = serde_json::to_string(&native_result)?;

        stdout.write_u32::<NativeEndian>(output_str.len() as u32)?;
        stdout.write_all(output_str.as_bytes())?;
        stdout.flush()?;
    }

    Ok(())
}

async fn get_reply(
    client: &mut DownloadManagerClient<tonic::transport::Channel>,
    config: &config::Config,
    input_str: &str,
) -> Result<NativeResult, Box<dyn std::error::Error>> {
    let native_message: NativeMessage = serde_json::from_str(&input_str)?;

    let output_str = match native_message {
        NativeMessage::FetchFileStates { query } => {
            let request = chrome_extension::GetFileStatesRequest { keys: query };
            let response = client.get_file_states(tonic::Request::new(request)).await?;
            NativeResult::FetchFileStates(response.get_ref().to_owned())
        }
        NativeMessage::RegisterFile {
            download_path,
            save_path,
            key,
        } => {
            let downloaded_file_path =
                PathBuf::from(&config.download_directory).join(&download_path);

            if save_path
                .iter()
                .any(|p| p.contains(&['/', '\\']) || p.contains(".."))
            {
                let err = anyhow::anyhow!("savePath must not contain slashes or dots.");
                Err(err)?;
            }
            let save_path_str = save_path.join("/");

            let save_dir = PathBuf::from(&config.save_file_directory)
                .join(save_path[..save_path.len() - 1].join("/"));
            let save_file_path = PathBuf::from(&config.save_file_directory).join(&save_path_str);

            std::fs::create_dir_all(save_dir)?;
            std::fs::rename(downloaded_file_path, save_file_path)?;

            let request = chrome_extension::RegisterFileRequest {
                path: save_path_str,
                key,
            };
            let response = client.register_file(tonic::Request::new(request)).await?;
            NativeResult::RegisterFile(response.get_ref().to_owned())
        }
    };

    Ok(output_str)
}

#[cfg(test)]
mod tests {
    use crate::chrome_extension::{
        self, download_manager_client::DownloadManagerClient, GetFileStatesReply,
    };
    use crate::native_messaging::{get_reply, NativeMessage, NativeResult};

    #[tokio::test]
    async fn test_file_states() -> Result<(), Box<dyn std::error::Error>> {
        let input_str = "{\"type\":\"fetch_file_states\",\"data\":{\"query\":[]}}";

        let mut client = DownloadManagerClient::connect("http://[::1]:50051").await?;

        let config_response = client
            .get_config(tonic::Request::new(chrome_extension::ConfigRequest {}))
            .await?;
        let config = config_response.get_ref();

        let reply = get_reply(&mut client, config.config.as_ref().unwrap(), input_str).await?;

        assert_eq!(
            reply,
            NativeResult::FetchFileStates(GetFileStatesReply { result: vec![] })
        );

        Ok(())
    }

    #[test]
    fn parse_register() {
        let input_str = "{\"type\":\"register_file\",\"data\":{\"downloadPath\":\"a\",\"savePath\":[\"b\",\"d\"],\"key\":\"c\"}}";
        let parsed: NativeMessage = serde_json::from_str(input_str).unwrap();

        assert_eq!(
            parsed,
            NativeMessage::RegisterFile {
                download_path: "a".to_string(),
                save_path: vec!["b".to_string(), "d".to_string()],
                key: "c".to_string()
            }
        );
    }
}
