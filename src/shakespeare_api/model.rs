use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TranslationResponse {
    pub contents: Translation,
}

#[derive(Deserialize, Debug)]
pub struct Translation {
    pub translated: String,
    pub translation: String,
}
