use std::path::PathBuf;

use enchant::Broker;
use srtlib::{ParsingError, Subtitles};
use thiserror::Error;

use crate::{FileProcessor, IterProcessing as _, ProcessingContext, SubProcess as _, SubtitleFile};

//TODO: test SymSpell
// crate : https://crates.io/crates/symspell / https://github.com/reneklacan/symspell
// dictionary : https://github.com/wolfgarbe/SymSpell/tree/master/SymSpell.FrequencyDictionary

// test spellbook/Nuspell
// https://crates.io/crates/spellbook

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

    #[error("failed to load dictionary with spellbokk")]
    SpellbookLoad(spellbook::ParseDictionaryError),

    #[error("frror while tried to add word in spellbook dictionary")]
    SpellbookAddWord(spellbook::ParseFlagError),

    #[error("failed to request dict with enchant Brocker")]
    EnchantRequestDict(String),
}

/// Checkspell of text subtitles content.
///
/// # Panics
///
/// Will panic if failed to load dictionary, change it to report.
pub fn spellcheck_subs(proc_ctx: ProcessingContext, files: &FileProcessor) {
    //let hunspell_fr = load_hunspell("fr_FR"); //TODO: do not hardcode

    //let fr_dict = zspell_load_fr_dic().unwrap(); //TODO: enable lang configuration
    //let fr_dict = spellbook_load_fr_dict().unwrap(); //TODO: enable lang configuration
    let fr_dict = enchant_load_fr_dict().unwrap();

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
    dict: &enchant::Dict, //&spellbook::Dictionary,
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
        // let errors = dict.check_indices(sub.text.as_str()).collect::<Vec<_>>();
        // if !errors.is_empty() {
        //     println!("for {} `{}`", sub.num, sub.text);
        //     errors
        //         .iter()
        //         .for_each(|(idx, str)| println!("\terror {idx}: `{str}`"));
        // }
        //eprintln!("process {} : {}", sub.num, sub.text.replace('\n', " "));

        //remove and split from  split with
        sub.text
            .split([' ', ',', '.', '\"', '\n', ':', '?', '!', '(', ')', '♪']) // '\'',
            .filter(|word| !word.is_empty())
            // Remove <i> and </i>,
            .map(|word| {
                word.trim()
                    .trim_start_matches("<i>")
                    .trim_start_matches('-')
                    .trim_start_matches("<i>")
                    .trim_end_matches("</i>")
            })
            .filter(|word| !word.is_empty())
            //.inspect(|word| eprintln!("\tcheck : '{word}'"))
            .filter(|word| !dict.check(word).unwrap())
            .for_each(|word| {
                //let mut suggestions = Vec::new();
                let suggestions = dict.suggest(word);
                let text_ctx = sub.text.replace('\n', " ");
                eprintln!(
                    "{}: {text_ctx}\n\t{word:?} is NOT in the dictionary. Did you mean {suggestions:?}?", sub.num
                );
            });
    });
    Ok(())
}

// const FR_AFF: &str = include_str!("/usr/share/hunspell/fr_FR.aff");
// const FR_DIC: &str = include_str!("/usr/share/hunspell/fr_FR.dic");
const FR_AFF: &str = include_str!("../dictionaries/fr/index.aff");
const FR_DIC: &str = include_str!("../dictionaries/fr/index.dic");

fn zspell_load_fr_dict() -> Result<zspell::Dictionary, SpellCheckError> {
    // Use the builder pattern to create our `Dictionary` object
    let dict: zspell::Dictionary = zspell::builder()
        .config_str(FR_AFF)
        .dict_str(FR_DIC)
        //.personal_str(HACK_CUSTOM_DICT)
        .build()
        .map_err(SpellCheckError::LoadDictionary)?;
    Ok(dict)
}

fn spellbook_load_fr_dict() -> Result<spellbook::Dictionary, SpellCheckError> {
    // Use the builder pattern to create our `Dictionary` object
    let mut dict =
        spellbook::Dictionary::new(FR_AFF, FR_DIC).map_err(SpellCheckError::SpellbookLoad)?;
    HACK_CUSTOM_DICT
        .iter()
        .try_for_each(|word| dict.add(word))
        .map_err(SpellCheckError::SpellbookAddWord)?;
    //TODO: push the error in context, not break loading
    Ok(dict)
    //    .personal_str(HACK_CUSTOM_DICT)
}

fn enchant_load_fr_dict() -> Result<enchant::Dict, SpellCheckError> {
    let mut spell_broker = Broker::new();
    // for dict in spell_broker.list_dicts() {
    //     println!("{:?}", dict);
    // }
    let dict = spell_broker
        .request_dict("fr_FR")
        .map_err(SpellCheckError::EnchantRequestDict)?;
    //TODO: add to Dict
    for word in HACK_CUSTOM_DICT {
        dict.add(word);
    }

    Ok(dict)
}

const HACK_CUSTOM_DICT: &[&str] = &[
    "Leonard",
    "Raj",
    "Rajesh",
    "Sheldon",
    "Cooper",
    "Howie",
    "Wolowitz",
    "Rostenkowski",
    "Priya",
    "Soyouz",
    "NASA",
    "Dave",
    "Roeger",
    "Bazinga",
    r"km/h",
    "Grinch",
    "Pasadena",
    "pastrami",
    "Nesquik",
    "Warlords",
    "Oreos",
    "Tropicana",
];

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
