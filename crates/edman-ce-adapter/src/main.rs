use native_messaging::main_loop;

mod native_messaging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = std::io::stdin().lock();
    let stdout = std::io::stdout().lock();

    main_loop(stdin, stdout).await?;

    Ok(())
}
