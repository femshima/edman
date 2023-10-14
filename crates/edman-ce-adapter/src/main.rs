use chrome_extension::download_manager_client::DownloadManagerClient;
use clap::{Args, Parser};
use manifest::{AppManifest, BrowserKind};
use native_messaging::main_loop;

mod installer;
mod manifest;
mod native_messaging;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[group(id = "input", multiple = false, conflicts_with = "browser")]
struct Cli {
    /// Install native messaging manifest for the extension registered to edman to the specified browser
    #[arg(group = "input", long)]
    install: Option<BrowserKind>,

    /// Uninstall native messaging manifest installed for edman in the specified browser
    #[arg(group = "input", long)]
    uninstall: Option<BrowserKind>,

    #[arg(group = "input", long)]
    manifest: Option<BrowserKind>,

    #[clap(flatten)]
    browser_arguments: BrowserArguments,
}

#[derive(Args)]
#[group(id = "browser")]
struct BrowserArguments {
    #[arg(hide = true)]
    origin: Option<String>,

    #[arg(hide = true, long)]
    parent_window: Option<i32>,
}

pub mod chrome_extension {
    #![allow(non_snake_case)]
    tonic::include_proto!("chrome_extension");
}
pub mod config {
    #![allow(non_snake_case)]
    tonic::include_proto!("config");
}

#[tokio::main]
async fn main() {
    if let Err(err) = main_procedure().await {
        let exit_code = write_error_log(err);
        std::process::exit(exit_code);
    }
}

async fn main_procedure() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(options) = cli.uninstall {
        return installer::uninstall(options);
    }

    let channel = transport::connect().await?;
    let mut client = DownloadManagerClient::new(channel);
    let config_response = client
        .get_config(tonic::Request::new(chrome_extension::ConfigRequest {}))
        .await?;
    let config = config_response.get_ref().config.as_ref().unwrap();

    if let Some(options) = cli.install {
        let manifest = AppManifest::new(&options, config)?;
        let manifest_str = serde_json::to_string_pretty(&manifest)?;
        installer::install(&options, manifest_str)?;
    } else if let Some(options) = cli.manifest {
        let manifest = AppManifest::new(&options, config)?;
        let manifest_str = serde_json::to_string_pretty(&manifest)?;
        println!("{}", manifest_str);
    } else {
        if let Some(ref origin) = cli.browser_arguments.origin {
            if !config.allowed_origins.contains(origin) {
                Err(anyhow::anyhow!("Origin \"{}\" is not allowed!", origin))?;
            }
        }

        let stdin = std::io::stdin().lock();
        let stdout = std::io::stdout().lock();

        main_loop(&mut client, config, stdin, stdout).await?;
    }

    Ok(())
}

fn write_error_log(err: Box<dyn std::error::Error>) -> i32 {
    use std::{
        fs::OpenOptions,
        io::Write,
        time::{SystemTime, UNIX_EPOCH},
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards!");
    let log_text = format!("{}: Error {:?}\n", now.as_millis(), err);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(utils::ce_adapter_error_log_path());
    let write_result = file.and_then(|mut error_log| error_log.write_all(log_text.as_bytes()));

    match write_result {
        Ok(()) => {
            eprintln!("{}", log_text);
            -1
        }
        Err(err) => {
            eprintln!(
                "Cannot open error log!\nError:\n{}\n\nThe original error:\n{}",
                err, log_text
            );
            -2
        }
    }
}
