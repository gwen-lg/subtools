//! A command line utilities to test and use subtools functionalities.

use anyhow::Context;
use clap::Parser;
use std::{env, ffi::OsString, path::PathBuf};
use subtools::{FileProcessor, SubtitleFile};

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
    files_processor.subtitle_files().for_each(|file| {
        match SubtitleFile::try_from(file.as_path()) {
            Ok(sub_file) => {
                eprintln!("{file:?} is recognized as {}", sub_file.format());
            }
            Err(err) => {
                eprintln!("{file:?} is not a recognized subtitle file : {err:?}");
            }
        }
    });
    Ok(())
}
