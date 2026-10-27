//! Interface language never changes database identifiers or user-entered values.
use serde::{Deserialize, Serialize};
use std::cell::Cell;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Ru,
    De,
    En,
}
impl Language {
    pub const ALL: [Self; 3] = [Self::Ru, Self::De, Self::En];
    pub fn name(self) -> &'static str {
        match self {
            Self::Ru => "Русский",
            Self::De => "Deutsch",
            Self::En => "English",
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Self::Ru => "ru",
            Self::De => "de",
            Self::En => "en",
        }
    }
}
thread_local! {static CURRENT:Cell<Language>=const{Cell::new(Language::Ru)};}
pub fn set(language: Language) {
    CURRENT.set(language);
}
pub fn current() -> Language {
    CURRENT.get()
}
pub fn tr(text: &str) -> &str {
    #[derive(Deserialize)]
    struct Translation {
        de: String,
        en: String,
    }
    static CATALOG: std::sync::OnceLock<std::collections::BTreeMap<String, Translation>> =
        std::sync::OnceLock::new();
    let catalog = CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/ui.json"))
            .expect("validated interface catalog")
    });
    match (current(), catalog.get(text)) {
        (Language::De, Some(t)) => &t.de,
        (Language::En, Some(t)) => &t.en,
        _ => text,
    }
}
