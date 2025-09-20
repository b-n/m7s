mod api_client;
mod config;
mod error;
mod logging;

use error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::init_logging();

    let config = config::config()?;

    let mut client = api_client::from_config(&config).await?;

    let kinds = client.get_kinds().await?;

    println!("Available kinds: {kinds:?}");

    Ok(())
}
