use std::str;

use super::SubtitleLineDecoder;
use subtile::{time::TimeSpan, webvtt::TimePointVtt};

///Wip + TODO
pub struct VobSubDecoder {}
impl VobSubDecoder {
    /// Create a `VobSub` subtitle line Decoder.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub fn new(index_header: &[u8]) -> Self {
        let index_header = str::from_utf8(index_header).unwrap();
        println!("VobSub idx:{index_header}");
        Self {}
    }
}
impl SubtitleLineDecoder for VobSubDecoder {
    fn push_sub_line(&mut self, time: TimeSpan, content: &[u8]) {
        let content_size = content.len();
        println!(
            "VobSub frame `{}`: {content_size}",
            TimePointVtt::from(time.start)
        );
    }
}
