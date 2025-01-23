//! A command line utility use subtools functionalities.

use anyhow::Context as _;
use clap::Parser;
use std::{
    borrow::Borrow,
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};
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

    let working_path = get_working_path(&args)?;
    let files_processor = FileProcessor::from_path(working_path);
    files_processor.subtitle_files().for_each(check_file);
    Ok(())
}

fn check_file<F>(file: F)
where
    F: Borrow<Path>,
{
    let file = file.borrow();
    match SubtitleFile::try_from(file) {
        Ok(subfile) => {
            eprintln!("{file:?} is recognized as {}", subfile.format());
        }
        Err(err) => {
            eprintln!("{file:?} is not a recognized subtitle file : {err:?}");
        }
    }
}

// get a working path from args or current dir.
fn get_working_path(args: &Cli) -> Result<PathBuf, anyhow::Error> {
    let in_path = if let Some(path) = &args.path {
        PathBuf::from(path)
    } else {
        env::current_dir().context("Failed to access to current directory")?
    };
    Ok(in_path)
}
