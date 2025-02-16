//! TODO

use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

// TODO: work context as struct than carry some object than implement trait, like Progress, sub_process_creator, a writer

/// Store the context elements.
pub struct ProcessingContext {
    logger: Box<dyn Write>,
    progress: Option<ProcessingProgress>,
    name: String,
    level: u8,
}

impl ProcessingContext {
    ///TODO: Init the progress info for this `ProcessingContext`.
    ///
    /// # Panics
    ///
    /// Will panic if a progress was already initialized.
    pub fn init_progress(&mut self, progress: ProcessingProgress) {
        self.progress
            .replace(progress)
            .ok_or(())
            .expect_err("A progress was already initialized.");
    }

    fn from_level(name: impl Into<String>, level: u8) -> Self {
        // Create a new `ProcessingContext` than write info in stderr
        let mut logger = ProcErrLogger::with_level(level);
        let name = name.into();
        writeln!(logger, "New processing created : {name}",).unwrap();
        Self {
            logger: Box::new(logger),
            progress: None,
            name,
            level: 0,
        }
    }
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
        Self::from_level(name, level)
    }
}

/// Define interaction with subprocess
pub trait SubProcess {
    /// Create a sub process context.
    fn create_sub_process(&self, name: impl Into<String>) -> ProcessingContext;
}

/// TODO
#[derive(Debug, Clone, Copy)]
pub enum Progress {
    Factor { current: f32 },
    Count { current: usize, end: usize },
}

/// TODO
#[derive(Debug)]
pub struct ProcessingProgress {
    progress: Progress,
}

/// TODO: add time management ?
impl ProcessingProgress {
    /// Create a `ProcessingProgress` for a number of elements.
    #[must_use]
    pub const fn from_count(nb_element: usize) -> Self {
        Self {
            progress: Progress::Count {
                current: 0,
                end: nb_element,
            },
        }
    }

    /// Create a `ProcessingProgress` for a factor management.
    #[must_use]
    pub const fn factor() -> Self {
        Self {
            progress: Progress::Factor { current: 0. },
        }
    }

    /// Get the progress, can be used for display
    #[must_use]
    pub const fn progress(&self) -> Progress {
        self.progress
    }

    /// Indicate than the process is finished
    #[must_use]
    pub fn is_finished(&self) -> bool {
        match self.progress {
            Progress::Factor { current } => current >= 1.,
            Progress::Count { current, end } => current >= end,
        }
    }

    /// Update the progress factor.
    ///
    /// # Panics
    ///
    /// Will panic if called on a [`Progress::Count`].
    pub fn update_factor(&mut self, new_current: f32) {
        match &mut self.progress {
            Progress::Factor { current } => *current = new_current,
            Progress::Count { .. } => panic!("shouldn't be call"),
        }
    }

    /// Update the progress value for [`Progress::Count`].
    ///
    /// # Panics
    ///
    /// Will panic if called on a [`Progress::Factor`].
    pub fn update_progress(&mut self, new_progress: usize) {
        match &mut self.progress {
            Progress::Factor { .. } => panic!("shouldn't be call"),
            Progress::Count { current, .. } => *current = new_progress,
        }
    }

    /// Increment the progress value for [`Progress::Count`].
    ///
    /// # Panics
    ///
    /// Will panic if called on a [`Progress::Factor`].
    pub fn increment_progress(&mut self) {
        match &mut self.progress {
            Progress::Factor { .. } => panic!("shouldn't be call"),
            Progress::Count { current, .. } => *current += 1,
        }
    }
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
        ProcessingContext::from_level(name, 0)
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

/// Create an iterator Processing
pub trait IterProcessing: Iterator {
    // ExactSizeIterator

    /// Add a [`ProcessingContext`] to an iterator.
    /// This allow to auto-magically handle progress
    fn process_context(
        self,
        ctx: impl SubProcess,
        name: impl Into<String>,
    ) -> ProcessingContextIter<Self>
    where
        Self: std::marker::Sized,
    {
        let mut sub_context = ctx.create_sub_process(name);
        //let nb_element = self.len(); // for ExactSizeIterator
        let (nb_element, _) = self.size_hint();

        sub_context.init_progress(ProcessingProgress::from_count(nb_element));
        ProcessingContextIter {
            iter: self,
            context: Rc::new(RefCell::new(sub_context)),
        }
    }

    /// Include a [`ProcessingContext`] into the iteraotr
    fn include_context(self, ctx: Rc<RefCell<ProcessingContext>>) -> ProcessingContextIter<Self>
    where
        Self: std::marker::Sized,
    {
        let (nb_element, _) = self.size_hint();

        ctx.borrow_mut()
            .init_progress(ProcessingProgress::from_count(nb_element));
        ProcessingContextIter {
            iter: self,
            context: ctx,
        }
    }
}

impl<U> IterProcessing for U where U: Iterator {}

/// TODO:
pub struct ProcessingContextIter<Iter> {
    iter: Iter,
    context: Rc<RefCell<ProcessingContext>>,
}

impl<Iter> Iterator for ProcessingContextIter<Iter>
where
    Iter: Iterator,
{
    type Item = (Rc<RefCell<ProcessingContext>>, Iter::Item); // (&'a mut ProcessingContext,

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(progress) = self.context.borrow_mut().progress.as_mut() {
            //TODO: update end, size_hint can be incorrect
            progress.increment_progress();
        }
        self.iter.next().map(move |val| (self.context.clone(), val))
    }
}
