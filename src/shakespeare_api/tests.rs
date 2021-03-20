use crate::shakespeare_api::model::{Translation, TranslationResponse};

const SAMPLE: &str = include_str!("sample.json");

const OUTPUT: &str = "Thee did giveth mr. Tim a hearty meal,  but unfortunately what he did doth englut did maketh him kicketh the bucket.";

#[test]
fn deserialize_service_response() {
    let result = serde_json::from_str::<TranslationResponse>(SAMPLE);
    assert!(result.is_ok());

    let TranslationResponse {
        contents: Translation {
            translated,
            translation,
        },
    } = result.unwrap();

    assert_eq!(translation, "shakespeare");
    assert_eq!(translated, OUTPUT);
}

#[cfg(feature = "api_tests")]
const SERVICE_URL: &str = "https://api.funtranslations.com/translate/shakespeare.json";

#[cfg(feature = "api_tests")]
const INPUT: &str = "You gave Mr. Tim a hearty meal, but unfortunately what he ate made him die.";

#[cfg(feature = "api_tests")]
#[tokio::test]
async fn call_service() {
    use crate::services::translation::TranslationService;
    use crate::shakespeare_api::ShakespeareService;
    use reqwest::{Client, Url};

    let url = Url::parse(SERVICE_URL).unwrap();
    let client = Client::new();

    let service = ShakespeareService::new(client, url);

    let result = service.attempt_translation(INPUT).await;

    assert_eq!(result, Ok(OUTPUT.to_string()));
}
