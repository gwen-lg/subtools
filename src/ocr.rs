use std::{fs::File, io::Write};

use crate::{file_processor::FileProcessor, subtitle_file::SubtitleFile};

pub fn ocr_subs(files: &FileProcessor) {
    files
        .subtitle_files()
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(|sub_file| sub_file.is_image())
        .for_each(|sub_file| {
            let mut path = sub_file.path().to_path_buf();
            path.set_extension("srt");

            if path.exists() {
                eprintln!("File '{path:?}' already exist, do not override with OCR.");
            } else {
                let mut file = File::create(path).unwrap();
                file.write_all("TODO".as_bytes()).unwrap();
            }
        });
}
