use super::FrameHandler;
use std::{
    io::{Seek, Write},
    mem,
};
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
    #[expect(clippy::missing_panics_doc)] //TODO: remove unwrap by return error, or by get str data
    pub fn new(mut idx_writer: Writer, sub_writer: Writer, idx_content: &[u8]) -> Self {
        //TODO: write timestamp and file pos, move to VobSubDecoder
        idx_writer
            .write_all("# VobSub index file, v7 (do not modify this line!)\n".as_bytes())
            .unwrap();
        idx_writer.write_all(idx_content).unwrap();
        //TODO: Fix `fr`
        idx_writer
            .write_all(b"langidx: 0\n\nid: fr, index: 0\n")
            .unwrap();

        //TODO: write langinfo.
        //  Example:
        // langidx: 0

        // id: fr, index: 0

        Self {
            idx_writer,
            sub_writer,
        }
    }
}

const PS_HEADER_TAG: [u8; 4] = [0x00, 0x00, 0x01, 0xba];
#[repr(C, packed)]
struct MpegPsHeader {
    pfx: [u8; 4], // 00 00 01 BA
    scr: [u8; 6],
    muxr: [u8; 3],
    stlen: u8,
}
impl MpegPsHeader {
    const fn from(c: u64) -> Self {
        // mpeg_ps_header_t ps;
        // memset(&ps, 0, sizeof(mpeg_ps_header_t));
        let mut scr = [0u8; 6];
        scr[0] = 0x40 | ((c >> 27) & 0x38) as u8 | 0x04 | ((c >> 28) & 0x03) as u8;
        scr[1] = (c >> 20) as u8;
        scr[2] = ((c >> 12) & 0xf8) as u8 | 0x04 | ((c >> 13) & 0x03) as u8;
        scr[3] = (c >> 5) as u8;
        scr[4] = ((c << 3) & 0xf8) as u8 | 0x04;
        scr[5] = 1;
        let muxr = [1u8, 0x89, 0xc3]; // just some value

        Self {
            pfx: PS_HEADER_TAG,
            scr,
            muxr,
            stlen: 0xf8,
        }
    }

    const fn bytes(&self) -> [u8; 14] {
        [
            self.pfx[0],
            self.pfx[1],
            self.pfx[2],
            self.pfx[3],
            self.scr[0],
            self.scr[1],
            self.scr[2],
            self.scr[3],
            self.scr[4],
            self.scr[5],
            self.muxr[0],
            self.muxr[1],
            self.muxr[2],
            self.stlen,
        ]
    }
}

#[repr(C, packed)]
struct MpegEsHeader {
    pfx: [u8; 3],  // 00 00 01
    stream_id: u8, // BD
    len: [u8; 2],
    flags: [u8; 2],
    hlen: u8,
    pts: [u8; 5],
    lidx: u8,
}
impl MpegEsHeader {
    const fn from(first: usize, c: u64, lidx: u8, padding: usize, size: usize) -> Self {
        let mut len = [0u8; 2];
        len[0] = ((first + 9) >> 8) as u8;
        len[1] = (first + 9) as u8;
        let mut pts = [0u8; 5];
        pts[0] = 0x20 | ((c >> 29) as u8 & 0x0e) | 0x01;
        pts[1] = (c >> 22) as u8;
        pts[2] = ((c >> 14) as u8 & 0xfe) | 0x01;
        pts[3] = (c >> 7) as u8;
        pts[4] = (c << 1) as u8 | 0x01;
        let mut hlen = 5;
        if (6 > padding) && (first == size) {
            hlen += padding as u8;
            len[0] = ((first + 9 + padding) >> 8) as u8;
            len[1] = (first + 9 + padding) as u8;
        }
        Self {
            pfx: [0, 0, 1],
            stream_id: 0xbd,
            len,
            flags: [0x81, 0x80],
            hlen,
            pts,
            lidx,
        }
    }
    //lidx volontarly ignored ?
    const fn bytes(&self) -> [u8; 14] {
        [
            self.pfx[0],
            self.pfx[1],
            self.pfx[2],
            self.stream_id,
            self.len[0],
            self.len[1],
            self.flags[0],
            self.flags[1],
            self.hlen,
            self.pts[0],
            self.pts[1],
            self.pts[2],
            self.pts[3],
            self.pts[4],
        ]
    }
    const fn lidx(&self) -> [u8; 1] {
        [self.lidx]
    }
}

//const PESPACKET_HEADER: &[u8] = &[0x00, 0x00, 0x01, 0xba];
const PADDING_DATA: &[u8] = &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

impl<Writer> FrameHandler for VobSubFrameHandler<Writer>
where
    Writer: Write + Seek,
{
    fn push_frame(&mut self, timestamp: u64, duration: Option<u64>, content: &[u8]) {
        let time_point = TimePoint::from_msecs(i64::try_from(timestamp).unwrap());
        let filepos = self.sub_writer.stream_position().unwrap();

        // Write
        writeln!(
            self.idx_writer,
            "timestamp: {}, filepos: {filepos:09x}",
            TimePointIdx::from(time_point),
        )
        .unwrap();

        // let content_size = content.len();
        // let (start_content, _) = content.split_at(30);
        // println!(
        //     "VobSub frame `{}` ({content_size}) : {start_content:X?}",
        //     time_point.to_secs()
        // );
        //

        let size = content.len();
        let padding = (2048
            - (size + mem::size_of::<MpegPsHeader>() + mem::size_of::<MpegEsHeader>()))
            & 2047;
        let first = if size + mem::size_of::<MpegPsHeader>() + mem::size_of::<MpegEsHeader>() > 2048
        {
            assert!(false); //Need to manage second content section
            2048 - mem::size_of::<MpegPsHeader>() - mem::size_of::<MpegEsHeader>()
        } else {
            size
        };
        let c = timestamp * 9 / 100_000;
        let ps = MpegPsHeader::from(c);

        let lidx = 0x20; //TODO: if !self.master { 0x20 } else { self.stream_id };
        let es = MpegEsHeader::from(first, c, lidx, padding, size);

        self.sub_writer.write_all(&ps.bytes()).unwrap();
        self.sub_writer.write_all(&es.bytes()).unwrap();
        if (0 < padding) && (6 > padding) && (first == size) {
            let (padding_data, _) = PADDING_DATA.split_at(padding);
            self.sub_writer.write_all(padding_data).unwrap();
        }
        //TODO write padding
        self.sub_writer.write_all(&es.lidx()).unwrap();

        //TODO: 25xu8
        // Clock and Ext : 46 bits
        // let bit_rate = take_bits(22u32); // Bit rate
        // let marker_bits = tag_bits(0b11, 2u8); // Marker bits.
        // let reserved = take_bits::<_, u8, u8, nom::error::Error<(&[u8], usize)>>(5u8); // Reserved.
        // let stuffing_length =
        //     take_bits::<_, usize, usize, nom::error::Error<(&[u8], usize)>>(3usize); // Number of bytes of stuffing.
        self.sub_writer.write_all(content).unwrap();
    }
}
