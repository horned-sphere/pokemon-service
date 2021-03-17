use std::fmt::{Display, Formatter};
use std::error::Error;
use futures::future::BoxFuture;

#[derive(Debug)]
pub enum TranslationError {
    ServiceUnavailable,
    TranslationFailed,
}

impl Display for TranslationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TranslationError::ServiceUnavailable => write!(f, "The translation service is currently unavailable.", ),
            TranslationError::TranslationFailed => write!(f, "The text could not be translated."),
        }
    }
}

impl Error for TranslationError {}

pub trait TranslationService {

    fn attempt_translation(&self, text: &str) -> BoxFuture<Result<String, TranslationError>>;

}