use super::FrameHandler;
use std::io::Write;
use subtile::{time::TimePoint, vobsub::TimePointIdx};

///Wip + TODO
pub struct VobSubFrameHandler<Writer> {
    idx_writer: Writer,
    sub_writer: Writer,
}
impl<Writer> VobSubFrameHandler<Writer>
where
    Writer: Write,
{
    /// Create a `VobSub` subtitle line Decoder.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub fn new(mut idx_writer: Writer, sub_writer: Writer, idx_content: &[u8]) -> Self {
        //TODO: write timestamp and file pos, move to VobSubDecoder
        idx_writer
            .write_all("# VobSub index file, v7 (do not modify this line!)".as_bytes())
            .unwrap();
        idx_writer.write_all(idx_content).unwrap();

        Self {
            idx_writer,
            sub_writer,
        }
    }
}
impl<Writer> FrameHandler for VobSubFrameHandler<Writer>
where
    Writer: Write,
{
    fn push_frame(&mut self, timestamp: u64, _duration: Option<u64>, content: &[u8]) {
        let time_point = TimePoint::from_msecs(i64::try_from(timestamp).unwrap());
        let content_size = content.len();
        let (start_content, _) = content.split_at(30);
        println!(
            "VobSub frame `{}` ({content_size}) : {start_content:X?}",
            time_point.to_secs()
        );
        self.sub_writer.write_all(content).unwrap();

        let filepos = 0; //TODO self.sub_writer

        // filepos: 000000000
        writeln!(
            self.idx_writer,
            "timestamp: {}, filepos: {filepos}",
            TimePointIdx::from(time_point),
        )
        .unwrap();
    }
}
