use thiserror::Error;

use crate::{FileProcessor, IterProcessing as _, ProcessingContext, SubProcess as _, SubtitleFile};

#[derive(Debug, Error)]
enum SpellcheckError {}

/// Checkspell of text subtitles content.
pub fn spellcheck_subs(proc_ctx: ProcessingContext, files: &FileProcessor) {
    files
        .subtitle_files()
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(SubtitleFile::is_text)
        .process_context(proc_ctx, "Spellcheck subtitles")
        .for_each(|(ctx, path)| {
            // if false {
            let res = spellcheck_text_subs(&ctx.borrow(), &path);
            if let Err(err) = res {
                ctx.borrow_mut().report_err(err.into());
            }
            // } else {
            //     text_subs_fixup(&ctx.borrow(), &path);
            // }
            //TODO: handling errors
        });
}

fn spellcheck_text_subs(
    proc_ctx: &ProcessingContext,
    file: &SubtitleFile,
) -> Result<(), SpellcheckError> {
    let filename = file.path();
    proc_ctx.create_sub_process(format!("Check {}", filename.display()));

    Ok(())
}
