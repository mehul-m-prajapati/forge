use clap::Parser;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use crate::log::LogLevel;

#[derive(Debug, Clone, Default, PartialEq, Eq, Display, AsRefStr, EnumString, EnumIter)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Command {
    #[default]
    Ask,
    Edit,
    Quit,
    Model,
}

impl Command {
    pub fn variants() -> Vec<String> {
        Self::iter().map(|m| format!("/{}", m.as_ref())).collect()
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// API Key to be used
    #[arg(short, long)]
    pub key: String,

    /// Model to be used
    #[arg(short, long)]
    pub model: Option<String>,

    /// Base URL to be used
    #[arg(short, long)]
    pub base_url: Option<String>,

    /// Log level to use
    #[arg(long)]
    pub log_level: Option<LogLevel>,
}
