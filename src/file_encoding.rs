use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write},
};

use crate::file_processor::{filter_text_subs, FileProcessor};

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

///TODO: report error in a context
pub fn convert_subs_to_utf8(files: &FileProcessor) {
    files
        .subtitle_files()
        .filter_map(filter_text_subs)
        .for_each(|file| {
            let reader = BufReader::new(file);
            let writer = BufWriter::new(File::create("TODO.srt").unwrap());
            convert_file_to_utf8(reader, writer);
        });
}

fn convert_file_to_utf8<R, W>(mut reader: R, mut writer: W)
where
    R: BufRead,
    W: Write,
{
    let has_utf8_bom = has_utf8_bom(&mut reader).unwrap();
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
    } else {
        //Write BOM UTF8 marker : EF BB BF
        writer.write_all(&UTF8_BOM).unwrap();

        //let mut line = String::with_capacity(128);
        reader
            .lines()
            .enumerate()
            .map(|(num_line, line)| {
                line.map_err(|err| format!("Fail to read line {num_line} : {err}"))
            })
            .try_for_each(|line| {
                //TODO: manage encoding
                //TODO: read all line to check all encoding errors
                let x = line.map_err(|err| io::Error::new(ErrorKind::Other, err))?;
                writer.write_all(x.as_bytes())
            })
            .unwrap();
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
    }
}
