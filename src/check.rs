use crate::{FileProcessor, IterProcessing, ProcessingContext, SubProcess, SubtitleFile};
use srtlib::{ParsingError, Subtitles};
use std::path::PathBuf;
use thiserror::Error;

type ReportEntry = String;

/// Define the report content for a data.
#[derive(Debug, Default)]
struct Report {
    entries: Vec<ReportEntry>,
}

impl Report {
    pub fn new() -> Self {
        Self {
            entries: Vec::default(),
        }
    }
    pub fn push(&mut self, error: ReportEntry) {
        self.entries.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Error)]
enum CheckError {
    #[error("failed to parse file `{file}' as srt/subrip format")]
    ParseSrt {
        #[source]
        source: ParsingError,
        file: PathBuf,
    },

    #[error("Some correction can be done in subtitle : {0:?}")]
    Report(Report),
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
    let report = basic_subline_check(sub_lines);
    if report.is_empty() {
        Ok(())
    } else {
        Err(CheckError::Report(report))
    }
}

//TODO: add format check (ex: subline number in srt)

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
    let mut report = Report::new();
    sublines.into_iter().enumerate().for_each(|(idx, subline)| {
        if subline.as_ref().is_empty() {
            report.push(format!("sub {idx} is empty"));
        } else {
            subline.as_ref().lines().for_each(|line| {
                if line.starts_with(char::is_whitespace) {
                    report.push(format!("sub {idx} start with withespace character"));
                }
                if line.ends_with(char::is_whitespace) {
                    report.push(format!("sub {idx} end with withespace character"));
                }
            });
        }
    });

    report
}

#[cfg(test)]
mod tests {
    use super::basic_subline_check;

    #[test]
    fn test_ok_sublines() {
        let subtitles = vec!["First", "second entry, bla", "third line", "end"];
        let report = basic_subline_check(subtitles);
        assert_eq!(report, None);
    }

    #[test]
    fn test_empty_sublines() {
        let subtitles = vec!["First", "second entry, bla", "", "end"];
        let report = basic_subline_check(subtitles);
        assert_eq!(report, Some(String::from("sub 2 is empty\n")));
    }

    #[test]
    fn test_start_whitespace() {
        let subtitles = vec!["First", "second entry,\n bla", "third line", "end"];
        let report = basic_subline_check(subtitles);
        assert_eq!(
            report,
            Some(String::from("sub 1 start with withespace character\n"))
        );
    }

    #[test]
    fn test_end_whitespace() {
        let subtitles = vec!["First", "second entry, \nbla", "third line", "end"];
        let report = basic_subline_check(subtitles);
        assert_eq!(
            report,
            Some(String::from("sub 1 end with withespace character\n"))
        );
    }
}
