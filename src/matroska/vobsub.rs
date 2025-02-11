use std::str;

use super::FrameHandler;
use subtile::time::TimePoint;

///Wip + TODO
pub struct VobSubFrameHandler {}
impl VobSubFrameHandler {
    /// Create a `VobSub` subtitle line Decoder.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub fn new(index_header: &[u8]) -> Self {
        let index_header = str::from_utf8(index_header).unwrap();
        println!("VobSub idx:{index_header}");
        Self {}
    }
}
impl FrameHandler for VobSubFrameHandler {
    fn push_frame(&mut self, timestamp: u64, _duration: Option<u64>, content: &[u8]) {
        let time_point = TimePoint::from_msecs(i64::try_from(timestamp).unwrap());
        let content_size = content.len();
        println!("VobSub frame `{}`: {content_size}", time_point.to_secs(),);
    }
}
