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
pub struct ProcessingCtx {
    log_writer: Box<dyn io::Write>,
}

impl ProcessingCtx {
    /// Create a new `ProcessingCtx` than write info in stderr
    #[must_use]
    pub fn with_stderr_writer() -> Self {
        Self {
            log_writer: Box::new(io::stderr()),
        }
    }

    /// Get access to writer to allow writing log
    pub fn writer(&mut self) -> &mut dyn io::Write {
        &mut self.log_writer
    }
}
