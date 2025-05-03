use crate::{
    FileProcessor, IterProcessing, ProcessingContext, SubProcess, SubtitleFile,
    regex::{RegexCheck, RegexOpReplace, RegexReplace},
};
use regex::Regex;
use srtlib::{ParsingError, Subtitle, Subtitles};
use std::{cell::RefCell, fmt::Display, path::PathBuf, rc::Rc, sync::LazyLock};
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

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.entries
            .iter()
            .try_for_each(|entry| writeln!(f, "- {entry}"))
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

    #[error("some correction can be done in subtitle : \n{0}")]
    Report(Report),

    #[error("failed to write to file `{file}' as srt/subrip format")]
    WriteToFile {
        #[source]
        source: ParsingError,
        file: PathBuf,
    },
}

/// Check text subtitles with predefined and custom rules.
pub fn check_subs(proc_ctx: ProcessingContext, files: &FileProcessor) {
    files
        .subtitle_files()
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(SubtitleFile::is_text)
        .process_context(proc_ctx, "Check subtitles")
        .for_each(|(ctx, path)| {
            if false {
                let res = check_text_subs(&ctx.borrow(), &path);
                if let Err(err) = res {
                    ctx.borrow_mut().report_err(err.into());
                }
            } else {
                let result = text_subs_fixup(&ctx.borrow(), &path);
                if let Err(err) = result {
                    println!("check_sub: {err:#?}");
                }
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

                //Check with regex
                //TODO: only for french
                static CA_CEDILLE: LazyLock<Regex> =
                    LazyLock::new(|| Regex::new("^Ca (.*)").unwrap());

                if CA_CEDILLE.is_match(line) {
                    report.push(format!("sub {idx} have `Ca` instead of `Ça`"));
                }
            });
        }
    });

    report
}

fn text_subs_fixup(proc_ctx: &ProcessingContext, file: &SubtitleFile) -> Result<(), CheckError> {
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

    let (subtitles, report) = basic_subline_fixup(subs);
    if report.is_empty() {
        Ok(())
    } else {
        let subtitles = Subtitles::new_from_vec(subtitles);
        let mut out_filename = file.path().to_path_buf();
        out_filename.set_extension("fixed.srt");

        //TODO: Write Utf-8 BOM, help for use with mkvtoolinx GUI
        subtitles
            .write_to_file(&out_filename, None)
            .map_err(|source| CheckError::WriteToFile {
                source,
                file: out_filename,
            })?;

        Err(CheckError::Report(report))
    }
}

// HACK Hardcode
// HARDCODED
// static REGEX_CHECK: LazyLock<RegexOpCheck> =
//     LazyLock::new(|| RegexOpCheck::try_from("todo").unwrap());

static REGEX_REPLACE: LazyLock<Vec<RegexOpReplace>> = LazyLock::new(|| {
    vec![
        // replace multiple space with one space
        RegexOpReplace::try_from((" {2,}", " ")).unwrap(),
        // Punctuation rules
        // Remove space before dot
        RegexOpReplace::try_from((r"\s+\.{1,1}", ".")).unwrap(), // "/ {2,}/g"
        // Remove space between point and number if point is the decimal separator
        RegexOpReplace::try_from((r"(?<f>\d). {2,}(?<s>\d)", "$f.$s")).unwrap(),
        // Remove space before comma
        RegexOpReplace::try_from((" ,", ",")).unwrap(),
        // Replace two apostrophe with quotation mark
        RegexOpReplace::try_from(("''", "\"")).unwrap(),
        RegexOpReplace::try_from(("^Ca (.*)", "Ça $1")).unwrap(), //TODO: check validity
        RegexOpReplace::try_from((" ca ", " ça ")).unwrap(),
        RegexOpReplace::try_from(("^II ", "Il ")).unwrap(),
        RegexOpReplace::try_from((". II ", ". Il ")).unwrap(), //TODO: regroup with previous ?
        RegexOpReplace::try_from(("^IIs ", "Ils ")).unwrap(),
        RegexOpReplace::try_from((". IIs ", ". Ils ")).unwrap(), //TODO: regroup with previous ?
        // Fix percent sign
        RegexOpReplace::try_from(("%/%", "%")).unwrap(),
        RegexOpReplace::try_from((r"\*/\*", "%")).unwrap(),
        RegexOpReplace::try_from(("°%", "%")).unwrap(),
        RegexOpReplace::try_from(("ïï", "ï")).unwrap(),
        // fin de ligne ` l` => ` !`
        RegexOpReplace::try_from((" [li]$", " !")).unwrap(),
        // Add missing space after comma
        RegexOpReplace::try_from((r",(?<after>\w)", ", $after")).unwrap(),
        // Specific
        RegexOpReplace::try_from(("4o", "40")).unwrap(),
    ]
});

//TODO: impl Iterator<Item = String> / use Cow<_, str> ?
fn basic_subline_fixup<Si>(sublines: Si) -> (Vec<Subtitle>, Report)
where
    Si: IntoIterator<Item = Subtitle>, //TODO: use typed Item (with contain but also info like idx, line number, etc)
{
    let report = Rc::new(RefCell::new(Report::new()));
    let sublines = sublines
        .into_iter()
        .filter_map(|subline| {
            if subline.text.is_empty() {
                report
                    .borrow_mut()
                    .push(format!("remove sub {} as empty", subline.num)); //TODO: display timing
                None
            } else {
                Some(subline)
            }
        })
        .enumerate()
        .map(|(idx, subline)| {
            let transform = subline.text.lines().map(|line| {
                if line.starts_with(char::is_whitespace) {
                    report
                        .borrow_mut()
                        .push(format!("sub {idx} start with withespace character"));
                }
                if line.ends_with(char::is_whitespace) {
                    report
                        .borrow_mut()
                        .push(format!("sub {idx} end with withespace character"));
                }
                let new_line = line.trim().to_string();

                REGEX_REPLACE.iter().for_each(|regex| {
                    if regex.check(new_line.as_str()) {
                        report.borrow_mut().push(format!("regex `{regex:?}` found"));
                    }
                });

                let new_line = REGEX_REPLACE.iter().fold(new_line, |val, regex| {
                    regex.replace(val.as_ref()).into_owned()
                });

                new_line.to_string()
            });

            let new_text = transform.fold(String::new(), |prev, line| {
                if prev.is_empty() {
                    line
                } else {
                    format!("{prev}\n{line}")
                }
            });

            Subtitle::new(idx, subline.start_time, subline.end_time, new_text)
        })
        .collect::<Vec<_>>();

    (sublines, report.take())
}

#[cfg(test)]
mod tests {
    use super::basic_subline_check;

    #[test]
    fn test_ok_sublines() {
        let subtitles = vec!["First", "second entry, bla", "third line", "end"];
        let report = basic_subline_check(subtitles);
        assert!(report.is_empty());
    }

    #[test]
    fn test_empty_sublines() {
        let subtitles = vec!["First", "second entry, bla", "", "end"];
        let report = basic_subline_check(subtitles);
        assert_eq!(
            format!("{report:?}"),
            String::from("Report { errors: [\"sub 2 is empty\"] }")
        );
    }

    #[test]
    fn test_start_whitespace() {
        let subtitles = vec!["First", "second entry,\n bla", "third line", "end"];
        let report = basic_subline_check(subtitles);
        assert_eq!(
            format!("{report:?}"),
            String::from("Report { errors: [\"sub 1 start with withespace character\"] }")
        );
    }

    #[test]
    fn test_end_whitespace() {
        let subtitles = vec!["First", "second entry, \nbla", "third line", "end"];
        let report = basic_subline_check(subtitles);
        assert_eq!(
            format!("{report:?}"),
            String::from("Report { errors: [\"sub 1 end with withespace character\"] }")
        );
    }
}
