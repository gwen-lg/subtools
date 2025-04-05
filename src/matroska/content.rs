use std::io::Read;

use matroska_demuxer::{ContentCompAlgo, ContentEncoding, ContentEncodingValue};

type Transformation = dyn Fn(Vec<u8>) -> Vec<u8>;

/// Implement a decompression
pub(crate) struct ContentDecoder {
    decoders: Vec<Box<Transformation>>,
}

impl ContentDecoder {
    pub fn new(encoding: Option<&[ContentEncoding]>) -> Self {
        let decoders: Vec<Box<Transformation>> = encoding
            .map(|encodings| {
                encodings
                    .iter()
                    .map(|encoding| {
                        assert!(encoding.scope() == 1); // 1 is for "Block", other are not managed
                        match encoding.encoding() {
                            ContentEncodingValue::Unknown => {
                                Box::new(pass_through) as Box<Transformation>
                            }
                            ContentEncodingValue::Compression(comp) => {
                                if comp.algo() == ContentCompAlgo::Zlib {
                                    if comp.settings().is_some() {
                                        panic!("Invalid decompress algo");
                                    }
                                    Box::new(decompress_zlib) as Box<Transformation>
                                } else {
                                    panic!(
                                        "Compression algorithm {:?} is not handled",
                                        comp.algo()
                                    );
                                }
                            }
                            ContentEncodingValue::Encryption(_) => todo!(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Self { decoders }
    }

    pub fn transform(&self, data: Vec<u8>) -> Vec<u8> {
        self.decoders
            .iter()
            .fold(data, |val: Vec<u8>, cur| cur(val))
    }
}

// Just a pass-through function
const fn pass_through(input: Vec<u8>) -> Vec<u8> {
    input
}

// Decompress fame data in Zlib format.
fn decompress_zlib(input: Vec<u8>) -> Vec<u8> {
    let mut decompressed = Vec::with_capacity(input.len() * 2);
    let mut zwriter = flate2::bufread::ZlibDecoder::new(input.as_slice());
    zwriter.read_to_end(&mut decompressed).unwrap();
    decompressed
}
