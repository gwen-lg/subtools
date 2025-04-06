use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use chardetng::EncodingDetector;
use encoding_rs::{CoderResult, Encoding};
use thiserror::Error;

use crate::{
    file_processor::FileProcessor, subtitle_file::SubtitleFile, IterProcessing, ProcessingContext,
};

pub const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

///TODO: report error in a context
#[allow(clippy::missing_panics_doc)]
pub fn convert_subs_to_utf8(proc_ctx: ProcessingContext, files: &FileProcessor) {
    files
        .subtitle_files()
        //TODO: ignore previous copy of old file
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(|sub_file| sub_file.is_text())
        .process_context(proc_ctx, "Convert subtitles file to utf-8")
        .for_each(|(ctx, sub_file)| {
            if let Err(err) = convert_file_to_utf8(&mut ctx.borrow_mut(), &sub_file) {
                ctx.borrow_mut().report_err(err.into());
            }
        });
}

//TODO: include file_name/path in `parent` Error/result
#[derive(Debug, Error)]
enum ConvertError {
    #[error("Failed to open file '{path}'")]
    OpenFile {
        #[source]
        source: io::Error,
        path: PathBuf,
    },

    #[error("Failed to fill buffer for file reading")]
    FillBuf(#[source] io::Error),

    #[error(
        "No Ascii was found in the start of the file content. Is this is ecpected, report an Issue"
    )]
    NoAscii,

    #[error("Encoding {encoding:?} is not managed")]
    EncodingNotManaged { encoding: String },

    #[error("Failed to create file '{path:?}'")]
    CreateFile {
        #[source]
        source: io::Error,
        path: PathBuf,
    },

    #[error("Failed to read line number '{num_line}'")]
    ReadLine {
        #[source]
        source: io::Error,
        num_line: usize,
    },

    ///TODO
    #[error("Failed to Write line {line_idx}:'{line_encoded}'")]
    WriteLine {
        #[source]
        source: io::Error,
        line_idx: usize,
        line_encoded: String,
    },

    #[error("Failed to rename file '{actual}' to '{new}'")]
    RenameFile {
        #[source]
        source: io::Error,
        actual: PathBuf,
        new: PathBuf,
    },

    #[error("Failed to write utf-8 BOM")]
    WriteUtf8BOM(#[source] io::Error),

    #[error("no enough data in reader to check BOM")]
    NoEnoughDataToCheckBOM,
}

fn convert_file_to_utf8(
    proc_ctx: &mut ProcessingContext,
    sub_file: &SubtitleFile,
) -> Result<(), ConvertError> {
    let filename = sub_file.filename().unwrap();
    writeln!(proc_ctx, "convert {filename:?}").unwrap();

    let file = File::open(sub_file.path()).map_err(|source| ConvertError::OpenFile {
        source,
        path: sub_file.path().to_path_buf(),
    })?;
    let mut reader = BufReader::new(file);

    let is_utf8 = match Encoding::for_bom(reader.fill_buf().map_err(ConvertError::FillBuf)?) {
        Some((encoding, size)) => {
            if encoding == encoding_rs::UTF_8 && size == 3 {
                true
            } else {
                return Err(ConvertError::EncodingNotManaged {
                    encoding: format!("{encoding:?}"),
                });
            }
        }
        None => false,
    };

    let has_utf8_bom = has_utf8_bom(&mut reader)?;
    assert!(is_utf8 == has_utf8_bom);
    if has_utf8_bom {
        // check
        reader
            .lines()
            .enumerate()
            .try_for_each(|(num_line, line)| {
                let _line = line.map_err(|source| ConvertError::ReadLine { source, num_line })?;

                Ok::<_, ConvertError>(())
            })?;
        writeln!(
            proc_ctx,
            "File `{:?}` is valid utf-8",
            sub_file.path().file_name()
        )
        .unwrap();
    } else {
        let out_filename = sub_file.gen_new_name("old");
        //TODO: check if old file already exist
        fs::rename(sub_file.path(), out_filename.as_path()).map_err(|source| {
            ConvertError::RenameFile {
                source,
                actual: sub_file.path().to_path_buf(),
                new: out_filename,
            }
        })?;
        let mut writer = BufWriter::new(File::create(sub_file.path()).map_err(|source| {
            ConvertError::CreateFile {
                source,
                path: sub_file.path().to_path_buf(),
            }
        })?);

        writer
            .write_all(&UTF8_BOM)
            .map_err(ConvertError::WriteUtf8BOM)?;

        let mut encoding_detector = EncodingDetector::new();
        let buf = reader.fill_buf().map_err(ConvertError::FillBuf)?;
        let has_no_ascii = encoding_detector.feed(buf, true);
        if !has_no_ascii {
            return Err(ConvertError::NoAscii);
        }
        assert!(has_no_ascii); //TODO: if only ascii : new more data, of utf8 compatible
        let encoding = encoding_detector.guess(None, true);

        //TODO: read all line to check all encoding errors
        let mut decoder = encoding.new_decoder();

        let mut line_idx = 1;
        let mut line_read = Vec::with_capacity(128);
        let mut line_encoded = String::with_capacity(256);
        while reader
            .read_until(b'\n', &mut line_read)
            .map_err(|source| ConvertError::ReadLine {
                source,
                num_line: line_idx,
            })?
            > 0
        {
            let (code_res, size_read, replacement_done) =
                decoder.decode_to_string(line_read.as_slice(), &mut line_encoded, false);
            assert!(code_res == CoderResult::InputEmpty);
            assert!(size_read == line_read.len());
            assert!(!replacement_done);

            //TODO: read all line to check all encoding errors
            writer
                .write_all(line_encoded.as_bytes())
                .map_err(|source| ConvertError::WriteLine {
                    source,
                    line_idx,
                    line_encoded: line_encoded.clone(),
                })?;

            line_idx += 1;
            line_encoded.clear();
            line_read.clear();
        }
        writeln!(
            proc_ctx,
            "File `{:?}` is converted to utf-8",
            sub_file.path().file_name()
        )
        .unwrap();

        // reader
        //     .lines()
        //     .enumerate()
        //     .map(|(num_line, line)| {
        //         line.map_err(|err| format!("Fail to read line {num_line} : {err}"))
        //     })
        //     .try_for_each(|line| {
        //         //TODO: manage encoding
        //         //TODO: read all line to check all encoding errors
        //         let x = line.map_err(|err| io::Error::new(ErrorKind::Other, err))?;
        //         writer.write_all(x.as_bytes())
        //     })
        //     .unwrap();
    }
    Ok(())
}

fn has_utf8_bom<R>(reader: &mut R) -> Result<bool, ConvertError>
where
    R: BufRead,
{
    let data = reader
        .fill_buf()
        .map_err(ConvertError::FillBuf)?
        .first_chunk::<3>()
        .ok_or(ConvertError::NoEnoughDataToCheckBOM)?;
    let has_uth8_bom = *data == UTF8_BOM;
    if has_uth8_bom {
        reader.consume(UTF8_BOM.len());
    }
    Ok(has_uth8_bom)
}

// fn read_line(mut prev_value: String, line: Result<String, String>) -> Result<String, String> {
//     prev_value.push_str(line?.as_str());
//     Ok(prev_value)
// }

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::{has_utf8_bom, UTF8_BOM};

    #[test]
    fn test_utf8_bom() {
        let mut bom_reader = BufReader::new(&UTF8_BOM[..]);
        assert!(has_utf8_bom(&mut bom_reader).unwrap());

        let mut small_reader = BufReader::new("test".as_bytes());
        assert!(!has_utf8_bom(&mut small_reader).unwrap());

        //TODO: test error of to small buffer
    }
}
