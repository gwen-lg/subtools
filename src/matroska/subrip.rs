use std::{io::Write, str};

use super::SubtitleLineDecoder;
use subtile::{srt, time::TimeSpan};

///TODO
pub struct SrtWriter<W: Write> {
    writer: W,
    last_line_index: u32,
}

impl<W: Write> SrtWriter<W> {
    /// Create a `SrtWriter`, to write extracted subtitle from mkv file.
    #[must_use]
    pub const fn new(writer: W) -> Self {
        Self {
            writer,
            last_line_index: 0,
        }
    }
}

impl<W: Write> SubtitleLineDecoder for SrtWriter<W> {
    fn push_sub_line(&mut self, time: TimeSpan, content: &[u8]) {
        self.last_line_index += 1;
        let line_text = str::from_utf8(content).unwrap();

        srt::write_line(
            &mut self.writer,
            self.last_line_index as usize,
            &time,
            line_text,
        )
        .unwrap();
    }
}
