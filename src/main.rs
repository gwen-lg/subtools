//! A command line utilities to test and use subtools functionalities.

mod commands;

use anyhow::Context;
use clap::Parser;
use commands::Commands;
use std::{env, ffi::OsString, path::PathBuf};
use subtools::{
    convert_subs_to_utf8, extract_subs, ocr_subs, AppContext, FileProcessor, SubProcess,
};

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
    let app_ctx = AppContext::new();
    let args = Cli::parse();

    //TODO: move into a file
    let in_path = if let Some(path) = args.path {
        PathBuf::from(path)
    } else {
        env::current_dir().context("Failed to access to current directory")?
    };

    let files_processor = FileProcessor::from_path(in_path);
    match args.command {
        Commands::ConvertToUtf8 => {
            let proc_ctx = app_ctx.create_sub_process("Convert subtitles files to UTF-8");
            convert_subs_to_utf8(proc_ctx, &files_processor);
        }
        Commands::Ocr => {
            let proc_ctx = app_ctx.create_sub_process("Convert subtitles binary files to UTF-8");
            ocr_subs(proc_ctx, &files_processor);
        }
        Commands::Extract => {
            let proc_ctx = app_ctx.create_sub_process("Extract subtitles from media file");
            extract_subs(proc_ctx, &files_processor);
        }
    }
    Ok(())
}
