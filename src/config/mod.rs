use clap::Parser;
use kube_client::config::Kubeconfig;
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
    /// Path to m7s config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Kubernetes context to use
    #[arg(long, value_name = "context")]
    context: Option<String>,

    /// The path to kubeconfig
    #[arg(long, value_name = "PATH", default_value = get_default_kube_config_path().into_os_string())]
    kube_config: PathBuf,

    /// File to edit
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Config {
    pub context: String,
    pub kube_config: Kubeconfig,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Kube config file not found: {0}")]
    MissingKubeConfig(PathBuf),
    #[error("Failed to read kube config: {0}")]
    KubeConfigReadError(#[from] kube_client::config::KubeconfigError),
    #[error("Invalid context: {0}")]
    InvalidContext(String),
}

pub fn parse() -> Result<Config, ConfigError> {
    let cli = CliConfig::parse();

    // Load kube config
    let kube_config_path = if cli.kube_config.is_relative() {
        env::current_dir()
            .expect("Could not get current directory")
            .join(cli.kube_config)
    } else {
        cli.kube_config
    };

    if !kube_config_path.exists() {
        Err(ConfigError::MissingKubeConfig(kube_config_path.clone()))?;
    }

    let kube_config = Kubeconfig::read_from(kube_config_path)?;

    let context =
        if let Some(ctx) = cli.context {
            kube_config.contexts.iter().find(|c| c.name == ctx).ok_or(
                ConfigError::InvalidContext(format!("'{ctx}' not found in kubeconfig")),
            )?;

            ctx
        } else {
            kube_config
                .current_context
                .clone()
                .ok_or(ConfigError::InvalidContext(
                    "Could not read current_context".to_string(),
                ))?
                .to_string()
        };

    Ok(Config {
        context,
        kube_config,
    })
}
