use serde::Deserialize;

/// Models the response from the Shakespeare translation API endpoint.
#[derive(Deserialize, Debug)]
pub struct TranslationResponse {
    pub contents: Translation,
}

#[derive(Deserialize, Debug)]
pub struct Translation {
    pub translated: String,
    pub translation: String,
}
