mod api_client;
mod app;
mod config;
mod error;
mod logging;

use error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::init_logging();

    let config = config::parse()?;

    let client = api_client::from_config(&config).await?;

    let mut app = app::App::new(client);
    let terminal = app.startup()?;
    let result = app.run(terminal).await;
    app.shutdown();

    result?;

    Ok(())
}
