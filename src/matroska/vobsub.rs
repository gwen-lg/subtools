use super::FrameHandler;
use static_assertions::assert_eq_size;
use std::{
    cmp::min,
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
        let mut scr = [0u8; 6];
        scr[0] = 0x40 | ((c >> 27) as u8 & 0x38) | 0x04 | ((c >> 28) as u8 & 0x03);
        scr[1] = (c >> 20) as u8;
        scr[2] = ((c >> 12) as u8 & 0xf8) | 0x04 | ((c >> 13) as u8 & 0x03);
        scr[3] = (c >> 5) as u8;
        scr[4] = ((c << 3) as u8 & 0xf8) | 0x04;
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
    const fn from(first: usize, c: u64, padding: usize, size: usize) -> Self {
        // lidx: u8,
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
            hlen += padding as u8; //TODO: check padding value
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
            lidx: 0, //Not write by this way
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
}

struct MpegEdHeader2 {
    data: [u8; 9],
}
impl MpegEdHeader2 {
    const fn from(first: usize, size: usize, padding: usize) -> Self {
        let mut hlen = 0;
        let padding = if (6 > padding) && (first == size) {
            hlen += padding as u8; //TODO: check padding value
            padding as u16
        } else {
            0_u16
        };
        let len = (first as u16 + 9 + padding).to_be_bytes();
        let data = [
            0x0_u8, // pfx[0]
            0x0_u8, // pfx[1]
            0x1_u8, // pfx[2]
            0xbd,   // stream_id
            len[0], // len[0]
            len[1], // len[1]
            0x81,   // flags[0]
            0x0,    // flags[1]
            hlen,   // hlen
        ];
        Self { data }
    }
    const fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

struct MpegEdHeader3 {
    data: [u8; 6],
}
impl MpegEdHeader3 {
    const fn from(padding: u16) -> Self {
        let len = u16::to_be_bytes(padding);
        let data = [
            0x0_u8, // pfx[0]
            0x0_u8, // pfx[1]
            0x1_u8, // pfx[2]
            0xbe,   // stream_id
            len[0], // len[0]
            len[1], // len[1]
        ];
        Self { data }
    }
    const fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

//const PESPACKET_HEADER: &[u8] = &[0x00, 0x00, 0x01, 0xba];
const PADDING_DATA: &[u8] = &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
const PACK_SIZE_MAX: usize = 2048 - mem::size_of::<MpegPsHeader>() - mem::size_of::<MpegEsHeader>();
const EMPTY: &[u8] = &[];

impl<Writer> FrameHandler for VobSubFrameHandler<Writer>
where
    Writer: Write + Seek,
{
    fn push_frame(&mut self, timestamp: u64, _duration: Option<u64>, content: &[u8]) {
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

        let size = content.len();
        let full_size = size + mem::size_of::<MpegPsHeader>() + mem::size_of::<MpegEsHeader>();
        let mut padding = (2048_usize.overflowing_sub(full_size)).0 & 2047;

        let c = timestamp * 90; //already converted ? * 9 / 100_000;
        let ps = MpegPsHeader::from(c);

        let (mut data, mut remaining) = content
            .split_at_checked(min(PACK_SIZE_MAX, content.len()))
            .unwrap();

        let first = usize::min(size, PACK_SIZE_MAX);
        let lidx = 0x20; //TODO: if !self.master { 0x20 } else { self.stream_id };

        let mut first_packet = true;

        while !data.is_empty() {
            self.sub_writer.write_all(&ps.bytes()).unwrap();
            if first_packet {
                let es_data = MpegEsHeader::from(first, c, padding, size);
                self.sub_writer.write_all(&es_data.bytes()).unwrap();
            } else {
                let es_data = MpegEdHeader2::from(first, size, padding);
                self.sub_writer.write_all(es_data.as_bytes()).unwrap();
            }

            // Write padding
            if (0 < padding) && (6 > padding) && (first == size) {
                let (padding_data, _) = PADDING_DATA.split_at(padding);
                self.sub_writer.write_all(padding_data).unwrap();
            }

            self.sub_writer.write_all(&[lidx]).unwrap();
            self.sub_writer.write_all(data).unwrap();

            // prepare next loop
            first_packet = false;
            if remaining.len() > 2048 - mem::size_of::<MpegPsHeader>() - 10 {
                let (new_data, new_remaining) = remaining.split_at(min(
                    2048 - mem::size_of::<MpegPsHeader>() - 10,
                    remaining.len(),
                ));
                data = new_data;
                remaining = new_remaining;
            } else {
                data = remaining;
                remaining = EMPTY;
            }
            if data.len() > 0 {
                padding = (2048 - (data.len() + 10 + mem::size_of::<MpegPsHeader>())) & 2047;
            }
        }
        if 6 <= padding {
            padding -= 6;
            let es = MpegEdHeader3::from(padding as u16);
            self.sub_writer.write_all(es.as_bytes()).unwrap();

            while 0 < padding {
                let todo = if 8 < padding { 8 } else { padding };
                let (padding_data, _) = PADDING_DATA.split_at(todo);
                self.sub_writer.write_all(padding_data).unwrap();
                padding -= todo;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp() {
        let ref_value = [0x44, 0x01, 0xC4, 0x87, 0x14];
        let time_value = 20437 * 90;
        let scr_0 =
            0x40 | ((time_value >> 27) as u8 & 0x38) | 0x04 | ((time_value >> 28) as u8 & 0x03);
        assert_eq!(ref_value[0], scr_0);
        let scr_1 = (time_value >> 20) as u8;
        assert_eq!(ref_value[1], scr_1);
        let scr_2 = ((time_value >> 12) & 0xf8) as u8 | 0x04 | ((time_value >> 13) & 0x03) as u8;
        assert_eq!(ref_value[2], scr_2);
        let scr_3 = (time_value >> 5) as u8;
        assert_eq!(ref_value[3], scr_3);
        let scr_4 = ((time_value << 3) & 0xf8) as u8 | 0x04;
        assert_eq!(ref_value[4], scr_4);
        // scr_5 = 1;
    }
}
