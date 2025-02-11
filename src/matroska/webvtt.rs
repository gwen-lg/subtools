use std::{io::Write, str};

use super::SubtitleLineDecoder;
use subtile::{time::TimeSpan, webvtt};

///TODO
pub struct WebvttWriter<W: Write> {
    writer: W,
}
impl<W: Write> WebvttWriter<W> {
    /// Create a `WebvttWriter` with header content from track `codec_private`
    #[must_use]
    #[allow(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub fn new(mut writer: W, track_codec_private: Option<&[u8]>) -> Self {
        writeln!(&mut writer, "WEBVTT\n").unwrap();

        if let Some(track_data) = track_codec_private {
            let codec_private = str::from_utf8(track_data).unwrap();
            writeln!(&mut writer, "{codec_private}").unwrap();
        }

        Self { writer }
    }
}
impl<W: Write> SubtitleLineDecoder for WebvttWriter<W> {
    fn push_sub_line(&mut self, time: TimeSpan, content: &[u8]) {
        let text = str::from_utf8(content).unwrap();
        webvtt::write_line(&mut self.writer, &time, text).unwrap();
    }
}
