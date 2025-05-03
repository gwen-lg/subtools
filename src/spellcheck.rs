use thiserror::Error;

use crate::{FileProcessor, IterProcessing as _, ProcessingContext, SubProcess as _, SubtitleFile};

#[derive(Debug, Error)]
enum SpellcheckError {}

/// Checkspell of text subtitles content.
pub fn spellcheck_subs(proc_ctx: ProcessingContext, files: &FileProcessor) {
    let hunspell_fr = load_hunspell("fr_FR"); //TODO: do not hardcode

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

    //TODO: move this in SubtitleFile
    let subs =
        Subtitles::parse_from_file(filename, None).map_err(|source| CheckError::ParseSrt {
            source,
            file: filename.into(),
        })?;

    subs.into_iter().for_each(|sub| {
        sub.text
            .split(' ')
            .map(|word| (word, hunspell.check(word)))
            .filter_map(|(word, result)| match result {
                CheckResult::MissingInDictionary => Some(word),
                CheckResult::FoundInDictionary => None,
            })
            .for_each(|res| println!("hunspell: {res}"));
    });
    Ok(())
}

//Wip
fn load_hunspell(lang: Lang) -> Hunspell {
    //let cargo_path = env!("");
    // let hunspell = Hunspell::new(
    // 	"../dictionaries/fr/index.aff",
    // 	"../dictionaries/fr/index.dic",
    // );
    const BASE_PATH: &str = "/usr/share/hunspell";
    let affpath = format!("{BASE_PATH}/{lang}.aff");
    let dicpath = format!("{BASE_PATH}/{lang}.dic");
    let hunspell = Hunspell::new(affpath.as_str(), dicpath.as_str());
    hunspell
}
