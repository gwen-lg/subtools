use std::path::PathBuf;

use srtlib::{ParsingError, Subtitles};
use thiserror::Error;
use zspell::Dictionary;

use crate::{FileProcessor, IterProcessing as _, ProcessingContext, SubProcess as _, SubtitleFile};

#[derive(Debug, Error)]
enum SpellCheckError {
    #[error("failed to parse file `{file}' as srt/subrip format")]
    ParseSrt {
        #[source]
        source: ParsingError,
        file: PathBuf,
    },

    #[error("failed to load dictionary")]
    LoadDictionary(#[source] zspell::Error),
}

/// Checkspell of text subtitles content.
///
/// # Panics
///
/// Will panic if failed to load dictionary, change it to report.
pub fn spellcheck_subs(proc_ctx: ProcessingContext, files: &FileProcessor) {
    //let hunspell_fr = load_hunspell("fr_FR"); //TODO: do not hardcode

    let fr_dict = load_fr_dic().unwrap(); //TODO: enable lang configuration

    files
        .subtitle_files()
        .filter_map(|path| SubtitleFile::try_from(path.as_path()).ok())
        .filter(SubtitleFile::is_text)
        .process_context(proc_ctx, "Spellcheck subtitles")
        .for_each(|(ctx, path)| {
            // if false {
            let res = spellcheck_text_subs(&ctx.borrow(), &path, &fr_dict);
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
    dict: &Dictionary,
) -> Result<(), SpellCheckError> {
    let filename = file.path();
    proc_ctx.create_sub_process(format!("Check {}", filename.display()));

    //TODO: move this in SubtitleFile
    let subs =
        Subtitles::parse_from_file(filename, None).map_err(|source| SpellCheckError::ParseSrt {
            source,
            file: filename.into(),
        })?;

    subs.into_iter().for_each(|sub| {
        let errors = dict.check_indices(sub.text.as_str()).collect::<Vec<_>>();
        if !errors.is_empty() {
            println!("for {} `{}`", sub.num, sub.text);
            errors
                .iter()
                .for_each(|(idx, str)| println!("\terror {idx}: `{str}`"));
        }
    });
    Ok(())
}

const FR_AFF: &str = include_str!("/usr/share/hunspell/fr_FR.aff");
const FR_DIC: &str = include_str!("/usr/share/hunspell/fr_FR.dic");

fn load_fr_dic() -> Result<Dictionary, SpellCheckError> {
    // Use the builder pattern to create our `Dictionary` object
    let dict: Dictionary = zspell::builder()
        .config_str(FR_AFF)
        .dict_str(FR_DIC)
        //.personal_str(HACK_CUSTOM_DICT)
        .build()
        .map_err(SpellCheckError::LoadDictionary)?;
    Ok(dict)
}

//Wip
// fn load_hunspell(lang: Lang) -> Hunspell {
//     //let cargo_path = env!("");
//     // let hunspell = Hunspell::new(
//     // 	"../dictionaries/fr/index.aff",
//     // 	"../dictionaries/fr/index.dic",
//     // );
//     const BASE_PATH: &str = "/usr/share/hunspell";
//     let affpath = format!("{BASE_PATH}/{lang}.aff");
//     let dicpath = format!("{BASE_PATH}/{lang}.dic");
//     let hunspell = Hunspell::new(affpath.as_str(), dicpath.as_str());
//     hunspell
// }
