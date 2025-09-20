mod config;
mod error;
mod kube_config;
mod logging;

use error::Error;

fn main() -> Result<(), Error> {
    logging::init_logging();

    let config = config::config()?;

    println!("Hello, world!");

    Ok(())
}
