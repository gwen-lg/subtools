use crate::{FileProcessor, IterProcessing, ProcessingContext, SubProcess, SubtitleFile};
use srtlib::{ParsingError, Subtitles};
use std::{fmt::Write, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
enum CheckError {
    #[error("failed to parse file `{file}' as srt/subrip format")]
    ParseSrt {
        #[source]
        source: ParsingError,
        file: PathBuf,
    },

    #[error("Some correction can be done in subtitle : {0}")]
    Report(String),
}

/// Check text subtitles with predefined and custom rules.
pub fn check_subs(proc_ctx: ProcessingContext, files: &FileProcessor) {
    files
        .subtitle_files()
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(SubtitleFile::is_text)
        .process_context(proc_ctx, "Check subtitles")
        .for_each(|(ctx, path)| {
            let res = check_text_subs(&ctx.borrow(), &path);
            if let Err(err) = res {
                ctx.borrow_mut().report_err(err.into());
            }
            //TODO: handling errors
        });
}

fn check_text_subs(proc_ctx: &ProcessingContext, file: &SubtitleFile) -> Result<(), CheckError> {
    let filename = file.path();
    proc_ctx.create_sub_process(format!("Check {filename:?}"));

    // let file = File::open(filename).map_err(|source| CheckError::OpeningFile {
    //     source,
    //     file: filename.into(),
    // })?;

    let subs =
        Subtitles::parse_from_file(filename, None).map_err(|source| CheckError::ParseSrt {
            source,
            file: filename.into(),
        })?;
    // let filereader = BufReader::new(file);
    // let lines = filereader.lines();
    //TODO: add other/configurable check

    let sub_lines = subs.into_iter().map(|s| s.text);
    if let Some(report) = basic_subline_check(sub_lines) {
        return Err(CheckError::Report(report));
    }
    Ok(())
}

//TODO: add format check (ex: subline number in srt)

//TODO: use a true type more complex
type Report = Option<String>;

/// Process some basic check to subline texte.
///
/// - check for empty subtitle line
/// - check for useless space at start or end of lines
///
/// check <i> with(out) </i>
/// TODO: add unit tests
fn basic_subline_check<S, T>(sublines: S) -> Report
where
    T: AsRef<str>,
    S: IntoIterator<Item = T>, //TODO: use typed Item (with contain but also info like idx, line number, etc)
{
    let mut report = String::new();
    sublines.into_iter().enumerate().for_each(|(idx, subline)| {
        if subline.as_ref().is_empty() {
            writeln!(&mut report, "sub {idx} is empty").unwrap();
        } else {
            subline.as_ref().lines().for_each(|line| {
                if line.starts_with(char::is_whitespace) {
                    writeln!(&mut report, "sub {idx} start with withespace character").unwrap();
                }
                if line.ends_with(char::is_whitespace) {
                    writeln!(&mut report, "sub {idx} end with withespace character").unwrap();
                }
            });
        }
    });

    if report.is_empty() {
        None
    } else {
        Some(report)
    }
}
