use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write},
};

use chardetng::EncodingDetector;
use encoding_rs::{CoderResult, Encoding};

use crate::{file_processor::FileProcessor, subtitle_file::SubtitleFile};

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

///TODO: report error in a context
pub fn convert_subs_to_utf8(files: &FileProcessor) {
    files
        .subtitle_files()
        //TODO: ignore previous copy of old file
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(|sub_file| sub_file.is_text())
        .for_each(|sub_file| {
            let file = File::open(sub_file.path())
                .inspect_err(|err| eprintln!("todo : {err:?}"))
                .ok();
            if let Some(file) = file {
                convert_file_to_utf8(&sub_file, file);
            } else {
                todo!()
            }
        });
}

fn convert_file_to_utf8(sub_file: &SubtitleFile, file: File) {
    let mut reader = BufReader::new(file);

    let is_utf8 = match Encoding::for_bom(reader.fill_buf().unwrap()) {
        Some((encoding, size)) => {
            if encoding == encoding_rs::UTF_8 && size == 3 {
                true
            } else {
                eprintln!("Encoding {encoding:?} is not managed");
                todo!()
            }
        }
        None => false,
    };

    let has_utf8_bom = has_utf8_bom(&mut reader).unwrap();
    assert!(is_utf8 == has_utf8_bom);
    if has_utf8_bom {
        // check
        reader
            .lines()
            .enumerate()
            .try_for_each(|(num_line, line)| {
                let _line = line.map_err(|err| {
                    io::Error::new(ErrorKind::Other, format!("Line {num_line} : {err}"))
                })?;

                Ok::<_, io::Error>(())
            })
            .unwrap();
        eprintln!("File `{:?}` is valid utf-8", sub_file.path().file_name());
    } else {
        let out_filename = sub_file.gen_new_name("old");
        //TODO: check if old file already exist
        fs::rename(sub_file.path(), out_filename).unwrap();
        let mut writer = BufWriter::new(File::create(sub_file.path()).unwrap());

        //Write BOM UTF8 marker : EF BB BF
        writer.write_all(&UTF8_BOM).unwrap();

        let mut encoding_detector = EncodingDetector::new();
        let buf = reader.fill_buf().unwrap();
        let has_no_ascii = encoding_detector.feed(buf, true);
        assert!(has_no_ascii); //TODO: if only ascii : new more data, of utf8 compatible
        let encoding = encoding_detector.guess(None, true);
        let mut decoder = encoding.new_decoder();

        let mut _line_idx = 1;
        let mut line_read = Vec::with_capacity(128);
        let mut line_encoded = String::with_capacity(256);
        while reader.read_until(b'\n', &mut line_read).unwrap() > 0 {
            let (code_res, size_read, replacement_done) =
                decoder.decode_to_string(line_read.as_slice(), &mut line_encoded, false);
            assert!(code_res == CoderResult::InputEmpty);
            assert!(size_read == line_read.len());
            assert!(!replacement_done);

            //TODO: read all line to check all encoding errors
            writer.write_all(line_encoded.as_bytes()).unwrap();
            //eprintln!("Line {line_idx} Utf8 error : {err}");

            //eprintln!("Line {line_idx}");
            _line_idx += 1;
            line_encoded.clear();
            line_read.clear();
        }
        eprintln!(
            "File `{:?}` is converted to utf-8",
            sub_file.path().file_name()
        );

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
}

fn has_utf8_bom<R>(reader: &mut R) -> Result<bool, io::Error>
where
    R: BufRead,
{
    let data = reader
        .fill_buf()?
        .first_chunk::<3>()
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "no enough data in reader to check BOM"))?;
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
