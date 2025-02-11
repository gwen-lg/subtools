//! `subtools` is a command line app to check and manipulate subtitles.

mod commands;
mod extract;
mod file_encoding;
mod file_processor;
pub mod matroska; //TODO: move in a crate (subtile ? or subtile-matroska ?)
mod ocr;
mod subtitle_file;

use std::{env, ffi::OsString, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use commands::Commands;
use extract::extract_subs;
use file_encoding::convert_subs_to_utf8;
use file_processor::FileProcessor;
use ocr::ocr_subs;

/// A CLI application to manipulate subtitles files.
#[derive(Debug, Parser)]
#[command(name = "sub_tools")]
#[command(about = "A command line tool to manipulate subtitles files with help of `subtile`", long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Can be a file, or a folder, if folder, it tried to process all compatible files of the folder.
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<OsString>,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    //TODO: move into a file
    let in_path = if let Some(path) = args.path {
        PathBuf::from(path)
    } else {
        env::current_dir().context("Failed to access to current directory")?
    };

    let files_processor = FileProcessor::from_path(in_path);
    match args.command {
        Commands::ConvertToUtf8 {} => {
            convert_subs_to_utf8(&files_processor);
        }
        Commands::Ocr {} => {
            ocr_subs(&files_processor);
        }
        Commands::Extract {} => {
            extract_subs(&files_processor);
        }
    }
    Ok(())
}
