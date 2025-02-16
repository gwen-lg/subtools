//! TODO

// /// Store the application context, to manage app processing information (report, progress, log, warning, error, ...)
// pub struct AppContext {
//     current_processing_ctxs: Vec<ProcessingCtx>
// }

// impl AppContext {
//     pub fn new() -> Self { Self{current_processing_ctxs: Vec::new()}}

//     pub fn start_processing(name: impl Into<String>) -> ProcessingCtx {

//     }
// }

use std::io;

/// A struct to manage procession context
pub struct ProcessingCtx<'p> {
    log_writer: Box<dyn io::Write>,
    parent: Option<&'p ProcessingCtx<'p>>,
    level: u8,
}

impl ProcessingCtx<'_> {
    /// Create a new `ProcessingCtx` than write info in stderr
    #[must_use]
    pub fn with_stderr_writer() -> Self {
        Self {
            log_writer: Box::new(io::stderr()),
            parent: None,
            level: 0,
        }
    }

    /// Get access to writer to allow writing log
    pub fn create_sub_process(&mut self) -> ProcessingCtx<'_> {
        let level = self.level + 1;
        ProcessingCtx {
            log_writer: Box::new(io::stderr()), //HACK
            parent: Some(&self),
            level,
        }
    }

    pub fn finish_sub_process(self) {
        let parent = self.parent.unwrap();
        parent.close(self);
    }

    fn close(&self, sub_process: ProcessingCtx<'_>) {
        drop(sub_process);
    }
}

impl io::Write for ProcessingCtx<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let tab = (0..self.level).fold(String::with_capacity(12), |mut val, _| {
            val.push_str("  ");
            val
        });
        write!(self.log_writer, "{}", tab)?;
        self.log_writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.log_writer.flush()
    }
}
