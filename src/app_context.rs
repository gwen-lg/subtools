//! TODO

use std::io::{self, Write};

// TODO: work context as struct than carry some object than implement trait, like Progress, sub_process_creator, a writer

/// Store the context elements.
pub struct ProcessingContext {
    logger: Box<dyn Write>,
    name: String,
    level: u8,
}

impl io::Write for ProcessingContext {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.logger.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.logger.flush()
    }
}

impl Drop for ProcessingContext {
    fn drop(&mut self) {
        writeln!(self.logger, "{} finished.", self.name).unwrap();
    }
}

impl SubProcess for ProcessingContext {
    /// Get access to writer to allow writing log
    fn create_sub_process(&self, name: impl Into<String>) -> ProcessingContext {
        let level = self.level + 1;
        Self {
            //app_ctx: self.app_ctx,
            logger: Box::new(ProcErrLogger::with_level(level)),
            //parent: Some(&self),
            name: name.into(),
            level,
        }
    }
}

/// Define interaction with subprocess
pub trait SubProcess {
    /// Create a sub process context.
    fn create_sub_process(&self, name: impl Into<String>) -> ProcessingContext;
}

pub struct ProgressInit {
    pub start: u32,
    pub end: u32,
}

/// TODO: describe
pub trait ProcessingProgress {
    //fn init(start: u32, end: u32);
    ///TODO: describe
    fn update_progress(&self, progress: f32);
    // /// Indicate than the processing is ended
    //fn end();
}

/// Store the application context, to manage app processing information (report, progress, log, warning, error, ...)
#[derive(Default)]
pub struct AppContext {
    current_processing_ctxs: Vec<ProcErrLogger>,
}

impl AppContext {
    /// Create a application context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_processing_ctxs: Vec::new(),
        }
    }
}

impl SubProcess for AppContext {
    fn create_sub_process(&self, name: impl Into<String>) -> ProcessingContext {
        // Create a new `ProcessingCtx` than write info in stderr
        ProcessingContext {
            logger: Box::new(ProcErrLogger {
                //app_ctx: self,
                log_writer: Box::new(io::stderr()),
                //parent: None,
                level_indent: String::new(),
            }),
            name: name.into(),
            level: 0,
        }
    }
}

impl ProcessingProgress for AppContext {
    fn update_progress(&self, _: f32) {
        todo!() // shouldn't be called
    }
}

/// A struct to manage procession context
pub struct ProcErrLogger {
    //app_ctx: &'a AppContext<'a>,
    log_writer: Box<dyn io::Write>,
    //parent: Option<&'p ProcessingCtx<'p>>,
    level_indent: String,
}

impl ProcErrLogger {
    fn with_level(level: u8) -> Self {
        let level_indent = (0..level).fold(String::with_capacity(12), |mut val, _| {
            val.push_str("  ");
            val
        });
        Self {
            log_writer: Box::new(io::stderr()),
            level_indent,
        }
    }
}

impl ProcessingProgress for ProcErrLogger {
    fn update_progress(&self, _: f32) {
        todo!() // shouldn't be called
    }
}

impl Drop for ProcErrLogger {
    fn drop(&mut self) {
        write!(self.log_writer, "{} finished.", self.level_indent).unwrap();
    }
}

// impl fmt::Write for ProcessingCtx {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         let msg = format_args!("{}-{}", self.level_indent, s);
//         self.log_writer.write_fmt(msg)
//     }
// }

impl io::Write for ProcErrLogger {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write!(self.log_writer, "{}", self.level_indent)?;
        self.log_writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.log_writer.flush()
    }
}
