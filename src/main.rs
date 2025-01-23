//! `subtools` is a command line app to check and manipulate subtitles.

mod file_processor;

use std::{env, ffi::OsString, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use file_processor::{filter_text_subs, FileProcessor};

/// A CLI application to manipulate subtitles files.
#[derive(Debug, Parser)]
#[command(name = "sub_tools")]
#[command(about = "A command line tool to manipulate subtitles files with help of `subtile`", long_about = None)]
struct Cli {
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
    files_processor
        .subtitle_files()
        .filter_map(filter_text_subs)
        .for_each(|file| {
            eprintln!("file : {file:?}");
        });
    Ok(())
}
