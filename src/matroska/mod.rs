//! specific code for matroska subtitle management

mod codec_id;

pub use codec_id::CodecId;

/// Define the interface to handle a frame from source (like matroska)
pub trait FrameHandler {
    /// Call with information/data on a subtitle line.
    fn push_frame(&mut self, timestamp: u64, duration: Option<u64>, content: &[u8]);
}
