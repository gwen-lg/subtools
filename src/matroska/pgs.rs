use super::{DumpInfo, FrameHandler};
use std::{fs::File, io::Write};
use subtile::{
    image::ToImage as _,
    pgs::{FrameConvertError, RleToImage, SegmentSplitter},
};

// Segment start Magic Number
const MAGIC_NUMBER: [u8; 2] = [0x50, 0x47];

/// A matroska Pgs decoder
#[derive(Default)]
pub struct PgsFrameHandler<Writer> {
    writer: Writer,
    dump_raw_frame: Option<DumpInfo>,
    dump_images: Option<DumpInfo>,
}
impl<Writer> PgsFrameHandler<Writer>
where
    Writer: Write,
{
    /// Create a `VobSub` subtitle line Decoder.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub const fn new(writer: Writer) -> Self {
        Self {
            writer,
            dump_raw_frame: None,
            dump_images: None,
        }
    }

    /// Enable the dump of raw frame.
    pub fn dump_raw_frame(&mut self, info: DumpInfo) {
        self.dump_raw_frame.replace(info);
    }

    /// Enable the dump of frame image
    pub fn dump_images(&mut self, info: DumpInfo) {
        self.dump_images.replace(info);
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

        if let Some(dump_info) = &self.dump_raw_frame {
            dump_raw_frame(dump_info, timestamp, content);
        }

        if let Some(dump_info) = &self.dump_images {
            dump_frame_image(dump_info, timestamp, content);
        }
    }
}

// Dump frame raw data in a file
fn dump_raw_frame(dump_info: &DumpInfo, timestamp: u64, content: &[u8]) {
    let mut file = File::create(format!(
        "{}_frame_{timestamp}{}.raw",
        dump_info.prefix,
        dump_info.lang_suffix()
    ))
    .unwrap();
    file.write_all(content).unwrap();
}

// Dump frame image in a file
fn dump_frame_image(dump_info: &DumpInfo, timestamp: u64, content: &[u8]) {
    let seg_splitter = SegmentSplitter::from(content);
    match seg_splitter.try_into() {
        Ok(rle_img) => {
            let image = RleToImage::new(&rle_img, |pix| pix).to_image();
            image
                .save(format!(
                    "{}{timestamp}{}.png",
                    dump_info.prefix,
                    dump_info.lang_suffix()
                ))
                .unwrap();
        }
        Err(FrameConvertError::NotAnImage) => {
            //TODO: manage end time
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }
}
