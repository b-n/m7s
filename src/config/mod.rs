use clap::Parser;
use kube_client::config::Kubeconfig;
use log::debug;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::{env, path::PathBuf};

fn get_default_kube_config_path() -> PathBuf {
    if let Ok(kube_config) = env::var("KUBECONFIG") {
        PathBuf::from(kube_config)
    } else if let Some(home_dir) = env::home_dir() {
        home_dir.join(".kube").join("config")
    } else {
        PathBuf::from("$HOME/.kube/config")
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliConfig {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(long, value_name = "context")]
    context: Option<String>,

    #[arg(long, value_name = "PATH", default_value = get_default_kube_config_path().into_os_string())]
    kube_config: PathBuf,
}

#[derive(Debug)]
pub struct Config {
    pub context: String,
    pub kube_config: Kubeconfig,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingKubeConfig(PathBuf),
    InvalidContext(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ConfigError::MissingKubeConfig(path) => {
                write!(f, "Kube config file not found: {}", path.display())
            }
            ConfigError::InvalidContext(ctx) => write!(f, "Invalid context: {ctx}"),
        }
    }
}

impl Error for ConfigError {}

pub fn config() -> Result<Config, ConfigError> {
    debug!("Loading config");
    let cli = CliConfig::parse();
    debug!("CLI Config: {cli:?}");

    // Load kube config
    let kube_config_path = if cli.kube_config.is_relative() {
        debug!("meow");
        env::current_dir()
            .expect("Could not get current directory")
            .join(cli.kube_config)
    } else {
        cli.kube_config
    };
    debug!("Kube config path: {}", kube_config_path.display());

    if !kube_config_path.exists() {
        Err(ConfigError::MissingKubeConfig(kube_config_path.clone()))?;
    }
    debug!("kube config path exists, loading...");

    let kube_config = Kubeconfig::read_from(kube_config_path).unwrap();

    let context = if let Some(ctx) = cli.context {
        ctx
    } else {
        kube_config
            .current_context
            .clone()
            .ok_or(ConfigError::InvalidContext(
                "Ensure a context is provided or selected in your kube configuration".to_string(),
            ))?
            .to_string()
    };
    debug!("Using context: {context}");

    Ok(Config {
        context,
        kube_config,
    })
}
