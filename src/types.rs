use std::fmt::{Display, Formatter};

#[derive(thiserror::Error, Debug)]
pub enum TypesError {
    #[error("Language '{0}' is not known")]
    UnknownLanguage(String),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Language {
    Python,
    Rust,
    JS,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "🐍"),
            Language::Rust => write!(f, "🦀"),
            Language::JS => write!(f, "🟨"),
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = TypesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let lower = value.to_lowercase();
        match lower.as_str() {
            "python" => Ok(Language::Python),
            "rust" => Ok(Language::Rust),
            "javascript" => Ok(Language::JS),
            _ => Err(TypesError::UnknownLanguage(value.to_string())),
        }
    }
}

impl TryFrom<&String> for Language {
    type Error = TypesError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
