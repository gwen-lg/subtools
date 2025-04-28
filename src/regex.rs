use std::{borrow::Cow, fmt::Debug};

use regex::{Error as RegexError, Regex, RegexBuilder};

pub trait RegexCheck {
    //TODO: add report/message for found
    fn check(&self, haystack: &str) -> bool;
}

pub trait RegexReplace {
    //TODO: add report/message for replace
    fn replace<'a>(&self, haystack: &'a str) -> Cow<'a, str>;
}

// enum RegexType {
//     CheckOnly,
//     CheckAndReplace,
// }

// Only check Regex
pub struct RegexOpCheck {
    regex: Regex,
}

impl TryFrom<&str> for RegexOpCheck {
    type Error = RegexError;

    fn try_from(pattern: &str) -> Result<Self, Self::Error> {
        let regex = RegexBuilder::new(pattern).multi_line(true).build()?;
        Ok(Self { regex })
    }
}

impl Debug for RegexOpCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Regex:\'{}\'", self.regex.as_str())
    }
}

impl RegexCheck for RegexOpCheck {
    fn check(&self, haystack: &str) -> bool {
        self.regex.is_match(haystack)
    }
}

pub struct RegexOpReplace {
    regex: Regex,
    replace: String,
}

impl TryFrom<(&str, &str)> for RegexOpReplace {
    type Error = RegexError;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let (pattern, replace) = value;
        let regex = RegexBuilder::new(pattern).multi_line(true).build()?;
        Ok(Self {
            regex,
            replace: replace.into(),
        })
    }
}

impl Debug for RegexOpReplace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Regex:\'{}\' to \'{}\'",
            self.regex.as_str(),
            self.replace
        )
    }
}

impl RegexCheck for RegexOpReplace {
    fn check(&self, haystack: &str) -> bool {
        self.regex.is_match(haystack)
    }
}

impl RegexReplace for RegexOpReplace {
    fn replace<'a>(&self, haystack: &'a str) -> Cow<'a, str> {
        self.regex.replace_all(haystack, self.replace.as_str())
    }
}
