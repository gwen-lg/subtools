use std::{io::Write, str};

use super::{FrameHandler, frame_time_span};
use subtile::webvtt;

///TODO
pub struct WebvttWriter<W: Write> {
    writer: W,
}

impl<W: Write> WebvttWriter<W> {
    /// Create a `WebvttWriter` with header content from track `codec_private`
    #[must_use]
    #[expect(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub fn new(mut writer: W, track_codec_private: Option<&[u8]>) -> Self {
        writeln!(&mut writer, "WEBVTT\n").unwrap();

        if let Some(track_data) = track_codec_private {
            let codec_private = str::from_utf8(track_data).unwrap();
            writeln!(&mut writer, "{codec_private}").unwrap();
        }

        Self { writer }
    }
}

impl<W: Write> FrameHandler for WebvttWriter<W> {
    fn push_frame(&mut self, timestamp: u64, duration: Option<u64>, content: &[u8]) {
        let text = str::from_utf8(content).unwrap();
        let time = frame_time_span(timestamp, duration.expect("need duration"));
        webvtt::write_line(&mut self.writer, &time, text).unwrap();
    }
}
