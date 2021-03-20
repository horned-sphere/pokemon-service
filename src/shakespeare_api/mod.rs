mod model;
#[cfg(test)]
mod tests;

use crate::services::translation::{TranslationError, TranslationService};
use crate::shakespeare_api::model::TranslationResponse;
use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::{Client, Error, Url};
use tracing::{event, Level};

/// A Shakespearian translation service provided by the API at
/// https://funtranslations.com/api/shakespeare.
pub struct ShakespeareService {
    client: Client,
    url: Url,
}

impl ShakespeareService {
    /// # Arguments
    /// * `client` HTTP client for making requests to the remote API.
    /// * `url` The URL of the translation endpoint.
    pub fn new(client: Client, url: Url) -> Self {
        ShakespeareService { client, url }
    }
}

impl From<reqwest::Error> for TranslationError {
    fn from(_: Error) -> Self {
        TranslationError::ServiceUnavailable
    }
}

/// Form key for the translation requests.
const FORM_KEY: &str = "text";

/// Expected ID of the translation.
const EXPECTED: &str = "shakespeare";

impl TranslationService for ShakespeareService {
    fn attempt_translation<'a>(
        &'a self,
        text: &'a str,
    ) -> BoxFuture<'a, Result<String, TranslationError>> {
        async move {
            let ShakespeareService { client, url } = self;

            event!(Level::INFO, message = "Making query to Shakespeare translation API.", %url, %text);

            let form_data = [(FORM_KEY, text)];
            let response = client.post(url.clone()).form(&form_data).send().await?;

            let status = response.status();

            event!(Level::DEBUG, message = "Received response from Shakespeare translation service.", %status);

            if status.is_success() {
                match response.json::<TranslationResponse>().await {
                    Ok(translated) if translated.contents.translation == EXPECTED => {
                        Ok(translated.contents.translated)
                    }
                    Ok(translated) => {
                        event!(Level::ERROR, message = "The translation service returned an unexpected translation.",
                            translation = %translated.contents.translation);
                        Err(TranslationError::TranslationFailed)
                    }
                    Err(error) => {
                        event!(Level::ERROR, message = "The translations serviced failed to translate the text.", %error, %status);
                        Err(TranslationError::TranslationFailed)
                    },
                }
            } else {
                event!(Level::ERROR, message = "Unanticipated response from Shakespeare translation service.", %status);
                Err(TranslationError::ServiceUnavailable)
            }
        }
        .boxed()
    }
}
