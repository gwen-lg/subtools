//! `subtools` is a library line app to check and manipulate subtitles.
//! A command line utilities is also provided.

mod file_encoding;
mod file_processor;
mod ocr;
mod subtitle_file;

pub use file_encoding::convert_subs_to_utf8;
pub use file_processor::FileProcessor;
pub use ocr::ocr_subs;
pub use subtitle_file::SubtitleFile;
