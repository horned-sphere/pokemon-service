mod model;

use reqwest::{Client, Url, Error};
use crate::services::translation::{TranslationService, TranslationError};
use futures::future::BoxFuture;
use futures::FutureExt;
use crate::shakespeare_api::model::TranslationResponse;

pub struct ShakespeareService {
    client: Client,
    url: Url,
}

impl ShakespeareService {

    pub fn new(client: Client,
           url: Url) -> Self {
        ShakespeareService{
            client,
            url
        }
    }

}

impl From<reqwest::Error> for TranslationError {
    fn from(_: Error) -> Self {
        TranslationError::ServiceUnavailable
    }
}

const EXPECTED: &str = "shakespeare";

impl TranslationService for ShakespeareService {
    fn attempt_translation<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<String, TranslationError>> {
        async move {
            let ShakespeareService { client, url } = self;
            let response = client.post(url.clone())
                .body(format!("text={}", text))
                .send()
                .await?;
            let status = response.status();
            if status.is_success() {
                match response.json::<TranslationResponse>().await {
                    Ok(translated) if translated.contents.translation == EXPECTED => {
                        Ok(translated.contents.translation)
                    }
                   _ => Err(TranslationError::TranslationFailed),
                }

            } else {
                Err(TranslationError::ServiceUnavailable)
            }
        }.boxed()
    }
}