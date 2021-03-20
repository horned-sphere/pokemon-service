mod model;
#[cfg(test)]
mod tests;

use crate::services::translation::{TranslationError, TranslationService};
use crate::shakespeare_api::model::TranslationResponse;
use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::{Client, Error, Url};

pub struct ShakespeareService {
    client: Client,
    url: Url,
}

impl ShakespeareService {
    pub fn new(client: Client, url: Url) -> Self {
        ShakespeareService { client, url }
    }
}

impl From<reqwest::Error> for TranslationError {
    fn from(_: Error) -> Self {
        TranslationError::ServiceUnavailable
    }
}

const FORM_KEY: &str = "text";
const EXPECTED: &str = "shakespeare";

impl TranslationService for ShakespeareService {
    fn attempt_translation<'a>(
        &'a self,
        text: &'a str,
    ) -> BoxFuture<'a, Result<String, TranslationError>> {
        async move {
            let ShakespeareService { client, url } = self;
            let form_data = [(FORM_KEY, text)];
            let response = client.post(url.clone()).form(&form_data).send().await?;
            let status = response.status();
            if status.is_success() {
                match response.json::<TranslationResponse>().await {
                    Ok(translated) if translated.contents.translation == EXPECTED => {
                        Ok(translated.contents.translated)
                    }
                    _ => Err(TranslationError::TranslationFailed),
                }
            } else {
                Err(TranslationError::ServiceUnavailable)
            }
        }
        .boxed()
    }
}
