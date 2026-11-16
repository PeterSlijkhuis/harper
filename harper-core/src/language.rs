use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

/// The natural language a document is written in.
/// [`crate::Dialect`] only applies when this is [`Language::English`].
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Display,
    EnumString,
    EnumIter,
)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    #[strum(serialize = "en")]
    English,
    #[serde(rename = "nl")]
    #[strum(serialize = "nl")]
    Dutch,
    #[serde(rename = "es")]
    #[strum(serialize = "es")]
    Spanish,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Language;

    #[test]
    fn codes_round_trip() {
        for (lang, code) in [
            (Language::English, "en"),
            (Language::Dutch, "nl"),
            (Language::Spanish, "es"),
        ] {
            assert_eq!(serde_json::to_string(&lang).unwrap(), format!("\"{code}\""));
            assert_eq!(
                serde_json::from_str::<Language>(&format!("\"{code}\"")).unwrap(),
                lang
            );
            assert_eq!(Language::from_str(code).unwrap(), lang);
            assert_eq!(lang.to_string(), code);
        }
    }
}
