//! A command line utility use subtools functionalities.

mod commands;

use anyhow::Context as _;
use clap::Parser;
use commands::Commands;
use std::{env, ffi::OsString, path::PathBuf};
use subtools::{
    AppContext, FileProcessor, SubProcess as _, check_subs, convert_subs_to_utf8, extract_subs,
    ocr_subs, spellcheck_subs,
};

/// A CLI application to manipulate subtitles files.
#[derive(Debug, Parser)]
#[command(version)]
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

    let working_path = get_working_path(&args)?;
    let files_processor = FileProcessor::from_path(working_path);
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
        Commands::Check => {
            let proc_ctx = app_ctx.create_sub_process("Check subtitles text with regex");
            check_subs(proc_ctx, &files_processor);
        }
        Commands::Spellcheck => {
            let proc_ctx = app_ctx.create_sub_process("Spellcheck of subtitles text");
            spellcheck_subs(proc_ctx, &files_processor);
        }
    }
    Ok(())
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
