mod api_client;
mod app;
mod config;
mod error;
mod logging;

use error::Error;
use std::error::Error as _;

#[tokio::main]
async fn main() {
    logging::init_logging();

    match run().await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            if let Some(source) = e.source() {
                eprintln!("  {source}");
            }
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<(), Error> {
    let config = config::parse()?;

    let client = api_client::from_config(&config).await?;

    let mut app = app::App::new(client);
    let terminal = app.startup(config.file)?;
    let result = app.run(terminal).await;
    app.shutdown();

    result?;
    Ok(())
}
