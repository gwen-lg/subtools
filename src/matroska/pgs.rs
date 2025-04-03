use super::SubtitleLineDecoder;
use std::io::Write;
use subtile::pgs::SegmentTypeCode;

/// A matroska Pgs decoder
#[derive(Default)]
pub struct PgsDecoder<Writer> {
    writer: Writer,
}
impl<Writer> PgsDecoder<Writer>
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

impl<Writer> SubtitleLineDecoder for PgsDecoder<Writer>
where
    Writer: Write,
{
    fn push_sub_line(&mut self, timestamp: u64, _duration: Option<u64>, mut content: &[u8]) {
        // Segment start Magic Number
        //const MAGIC_NUMBER: [u8; 2] = [0x50, 0x47];
        let mut sup_header: [u8; 10] = [0x50, 0x47, 0, 0, 0, 0, 0, 0, 0, 0];
        //let buffer = sup_header.as_mut_slice();
        //buffer[0..2] = []; //Magic Number
        //TODO: verify Timestamp is converted in ms, just go back to tick with mul by 90 ?
        let pts = u32::to_be_bytes(u32::try_from(timestamp * 90).unwrap());
        sup_header[2..6].copy_from_slice(&pts);

        while !content.is_empty() {
            self.writer.write_all(&sup_header).unwrap();

            let seg_size = u16::from_be_bytes(content[1..3].try_into().unwrap()) + 3; // 3 to take the header size into account
            SegmentTypeCode::try_from(content[0]).unwrap();

            //println!("\tWrite segment {seg_code}, size {seg_size}");
            let (buf_to_write, next) = content.split_at(seg_size as usize);
            self.writer.write_all(buf_to_write).unwrap();
            content = next;
        }
    }
}
