use futures::future::BoxFuture;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum TranslationError {
    /// The service could not produce a result for any reason other than the record not existing.
    ServiceUnavailable,
    /// It was not possibe to translate the text.
    TranslationFailed,
}

impl Display for TranslationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TranslationError::ServiceUnavailable => {
                write!(f, "The translation service is currently unavailable.",)
            }
            TranslationError::TranslationFailed => write!(f, "The text could not be translated."),
        }
    }
}

impl Error for TranslationError {}

/// A service to translate the descriptions for Pokemon species.
pub trait TranslationService {
    /// Attempt to translate a description.
    fn attempt_translation<'a>(
        &'a self,
        text: &'a str,
    ) -> BoxFuture<'a, Result<String, TranslationError>>;
}
