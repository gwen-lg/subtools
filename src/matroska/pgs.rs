use super::FrameHandler;
use std::io::Write;
use subtile::pgs::SegmentSplitter;

// Segment start Magic Number
const MAGIC_NUMBER: [u8; 2] = [0x50, 0x47];

/// A matroska Pgs decoder
#[derive(Default)]
pub struct PgsFrameHandler<Writer> {
    writer: Writer,
}
impl<Writer> PgsFrameHandler<Writer>
where
    Writer: Write,
{
    /// Create a `VobSub` subtitle line Decoder.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub const fn new(writer: Writer) -> Self {
        Self { writer }
    }
}

impl<Writer> FrameHandler for PgsFrameHandler<Writer>
where
    Writer: Write,
{
    fn push_frame(&mut self, timestamp: u64, _duration: Option<u64>, content: &[u8]) {
        let mut sup_header = [0_u8; 10];
        // Segment start Magic Number : [0x50, 0x47]
        sup_header[0..2].copy_from_slice(&MAGIC_NUMBER);
        //TODO: verify Timestamp is converted in ms, just go back to tick with mul by 90 ?
        let pts = u32::to_be_bytes(u32::try_from(timestamp * 90).unwrap());
        sup_header[2..6].copy_from_slice(&pts);

        SegmentSplitter::from(content).for_each(|seg_buf| {
            self.writer.write_all(&sup_header).unwrap();
            self.writer.write_all(seg_buf.unwrap().buffer()).unwrap();
        });
    }
}
