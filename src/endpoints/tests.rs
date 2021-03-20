use crate::endpoints::ServiceError;
use crate::model::PokemonData;
use crate::services::pokemon::{PokemonService, PokemonServiceError};
use crate::services::translation::{TranslationError, TranslationService};
use futures::future::{ready, BoxFuture};
use futures::FutureExt;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::sync::Arc;
use warp::{Rejection, Reply};

enum FakePokeService {
    Unavailable,
    Containing(HashMap<String, PokemonData>),
}

impl FakePokeService {
    fn with(name: &str, description: &str) -> Self {
        let mut map = HashMap::new();
        map.insert(
            name.to_string(),
            PokemonData {
                name: name.to_string(),
                description: description.to_string(),
            },
        );
        FakePokeService::Containing(map)
    }
}

impl PokemonService for FakePokeService {
    fn get_pokemon<'a>(
        &'a self,
        name: &'a str,
    ) -> BoxFuture<'a, Result<PokemonData, PokemonServiceError>> {
        ready(match self {
            FakePokeService::Unavailable => Err(PokemonServiceError::ServiceUnavailable),
            FakePokeService::Containing(map) => {
                if let Some(data) = map.get(name) {
                    Ok(data.clone())
                } else {
                    Err(PokemonServiceError::NoSuchPokemon(name.to_string()))
                }
            }
        })
        .boxed()
    }
}

enum FakeTranslationService {
    Unavailable,
    Fail,
    Succeed,
}

impl TranslationService for FakeTranslationService {
    fn attempt_translation<'a>(
        &'a self,
        text: &'a str,
    ) -> BoxFuture<'a, Result<String, TranslationError>> {
        ready(match self {
            FakeTranslationService::Unavailable => Err(TranslationError::ServiceUnavailable),
            FakeTranslationService::Fail => Err(TranslationError::TranslationFailed),
            FakeTranslationService::Succeed => Ok(text.to_uppercase()),
        })
        .boxed()
    }
}

#[tokio::test]
async fn combined_services_nominal() {
    let poke_service = Arc::new(FakePokeService::with("name", "A description."));
    let trans_service = Arc::new(FakeTranslationService::Succeed);

    let result = super::handle_request("name".to_string(), poke_service, trans_service).await;

    assert_eq!(
        result,
        Ok(PokemonData {
            name: "name".to_string(),
            description: "A DESCRIPTION.".to_string()
        })
    );
}

#[tokio::test]
async fn pokemon_service_unavailable() {
    let poke_service = Arc::new(FakePokeService::Unavailable);
    let trans_service = Arc::new(FakeTranslationService::Succeed);

    let result = super::handle_request("name".to_string(), poke_service, trans_service).await;

    assert_eq!(result, Err(ServiceError::ServiceUnavailable));
}

#[tokio::test]
async fn translations_unavailable() {
    let poke_service = Arc::new(FakePokeService::with("name", "A description."));
    let trans_service = Arc::new(FakeTranslationService::Unavailable);

    let result = super::handle_request("name".to_string(), poke_service, trans_service).await;

    assert_eq!(result, Err(ServiceError::ServiceUnavailable));
}

#[tokio::test]
async fn translation_failed() {
    let poke_service = Arc::new(FakePokeService::with("name", "A description."));
    let trans_service = Arc::new(FakeTranslationService::Fail);

    let result = super::handle_request("name".to_string(), poke_service, trans_service).await;

    assert_eq!(result, Err(ServiceError::TranslationFailed));
}

#[tokio::test]
async fn pokemon_not_found() {
    let poke_service = Arc::new(FakePokeService::with("name", "A description."));
    let trans_service = Arc::new(FakeTranslationService::Succeed);

    let result = super::handle_request("other".to_string(), poke_service, trans_service).await;

    assert_eq!(
        result,
        Err(ServiceError::NoSuchPokemon("other".to_string()))
    );
}

#[tokio::test]
async fn filter_good_request() {
    let poke_service = FakePokeService::with("name", "A description.");
    let trans_service = FakeTranslationService::Succeed;

    let filter = super::make_endpoint_filter(poke_service, trans_service);
    let result = warp::test::request()
        .path("/pokemon/name")
        .filter(&filter)
        .await;

    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(
        data,
        PokemonData {
            name: "name".to_string(),
            description: "A DESCRIPTION.".to_string()
        }
    );
}

#[tokio::test]
async fn filter_bad_request() {
    let poke_service = FakePokeService::with("name", "A description.");
    let trans_service = FakeTranslationService::Succeed;

    let filter = super::make_endpoint_filter(poke_service, trans_service);
    let result = warp::test::request()
        .path("/pokemon/other")
        .filter(&filter)
        .await;

    assert!(result.is_err());
    let reject: Rejection = result.err().unwrap();

    let err = reject.find::<ServiceError>();
    assert_eq!(err, Some(&ServiceError::NoSuchPokemon("other".to_string())));
}

#[test]
fn error_http_status_codes() {
    assert_eq!(
        ServiceError::ServiceUnavailable.into_response().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        ServiceError::TranslationFailed.into_response().status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        ServiceError::NoSuchPokemon("name".to_string())
            .into_response()
            .status(),
        StatusCode::NOT_FOUND
    );
}
