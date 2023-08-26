use chrome_extension::download_manager_client::DownloadManagerClient;
use clap::Parser;
use installer::InstallOptions;
use native_messaging::main_loop;

mod installer;
mod native_messaging;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(group = "input", long)]
    install: Option<InstallOptions>,

    #[arg(group = "input", long)]
    uninstall: Option<InstallOptions>,

    #[arg(group = "input")]
    origin: Option<String>,
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(options) = cli.uninstall {
        return installer::uninstall(options);
    }

    let mut client = DownloadManagerClient::connect("http://[::1]:50051").await?;
    let config_response = client
        .get_config(tonic::Request::new(chrome_extension::ConfigRequest {}))
        .await?;
    let config = config_response.get_ref().config.as_ref().unwrap();

    if let Some(options) = cli.install {
        installer::install(options, config)?;
    } else {
        let stdin = std::io::stdin().lock();
        let stdout = std::io::stdout().lock();

        main_loop(&mut client, config, stdin, stdout).await?;
    }

    Ok(())
}
