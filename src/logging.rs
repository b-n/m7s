use env_logger::{Builder, Env};
use std::env;

pub fn init_logging() {
    let target = env::var("LOG_TARGET").unwrap_or_else(|_| "m7s.log".to_string());

    let log_env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    let mut builder = Builder::from_env(log_env);

    if target != "stdout" {
        let log_file = std::fs::File::create("m7s.log").expect("Could not create m7s.log");
        builder.target(env_logger::Target::Pipe(Box::new(log_file)));
    }

    builder.init();
}
