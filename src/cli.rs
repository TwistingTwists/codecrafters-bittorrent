use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "snake_case")]
pub enum Commands {
    /// Decodes the serialized bencoded value into a printable format
    Decode {
        /// The serialized bencoded value to be decoded and further printed
        bencoded_value: String,
    },
    /// Parses a torrent file
    Info {
        /// The torrent file to be parsed and extracts information out of it
        torrent_file: PathBuf,
    },
}

