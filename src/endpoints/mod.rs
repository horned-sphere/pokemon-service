use crate::model::{ErrorMessage, PokemonData};
use crate::services::pokemon::{PokemonService, PokemonServiceError};
use crate::services::translation::{TranslationError, TranslationService};
use reqwest::StatusCode;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{event, Level};
use warp::reject::Reject;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

#[cfg(test)]
mod tests;

/// Create the warp filter for the single endpoint and execute it.
///
/// # Arguments
///
/// * `socket_addr` - Address to bind to.
/// * `pokemon_service` - A service implementation to get descriptions for Pokemon species.
/// * `translation_service`- A service implementation to transform the descriptions.
pub async fn run_server<Poke, Trans>(
    socket_addr: SocketAddr,
    pokemon_service: Poke,
    translation_service: Trans,
) where
    Poke: PokemonService + Send + Sync + 'static,
    Trans: TranslationService + Send + Sync + 'static,
{
    let endpoint =
        make_endpoint_filter(pokemon_service, translation_service).recover(handle_rejection);

    warp::serve(endpoint).run(socket_addr).await
}

fn make_endpoint_filter<Poke, Trans>(
    pokemon_service: Poke,
    translation_service: Trans,
) -> impl Filter<Extract = (PokemonData,), Error = Rejection> + Clone
where
    Poke: PokemonService + Send + Sync + 'static,
    Trans: TranslationService + Send + Sync + 'static,
{
    let shared_pokemon_service = Arc::new(pokemon_service);
    let shared_translation_service = Arc::new(translation_service);

    let pokemon_service_filter = warp::any().map(move || shared_pokemon_service.clone());
    let shared_translation_service = warp::any().map(move || shared_translation_service.clone());

    warp::path!("pokemon" / String)
        .and(pokemon_service_filter)
        .and(shared_translation_service)
        .and_then(|name, pokemon, trans| async move {
            handle_request(name, pokemon, trans)
                .await
                .map_err(warp::reject::custom)
        })
}

async fn handle_request<Poke, Trans>(
    name: String,
    pokemon_service: Arc<Poke>,
    translation_service: Arc<Trans>,
) -> Result<PokemonData, ServiceError>
where
    Poke: PokemonService,
    Trans: TranslationService,
{
    event!(Level::INFO, message = "Handling request.", %name);
    let mut response = pokemon_service.get_pokemon(name.as_str()).await?;
    response.description = translation_service
        .attempt_translation(response.description.as_str())
        .await?;
    Ok(response)
}

impl From<PokemonServiceError> for ServiceError {
    fn from(e: PokemonServiceError) -> Self {
        match e {
            PokemonServiceError::ServiceUnavailable => ServiceError::ServiceUnavailable,
            PokemonServiceError::NoSuchPokemon(name) => ServiceError::NoSuchPokemon(name),
        }
    }
}

impl From<TranslationError> for ServiceError {
    fn from(e: TranslationError) -> Self {
        match e {
            TranslationError::ServiceUnavailable => ServiceError::ServiceUnavailable,
            TranslationError::TranslationFailed => ServiceError::TranslationFailed,
        }
    }
}

/// Combined error type for the service (convertible to an HTTP error response).
#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceError {
    /// No Pokemon of that name exists (404).
    NoSuchPokemon(String),
    /// The description could not be translated (500).
    TranslationFailed,
    /// One of the services could not provide a reply (503).
    ServiceUnavailable,
}

impl ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::NoSuchPokemon(_) => StatusCode::NOT_FOUND,
            ServiceError::TranslationFailed => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NoSuchPokemon(name) => {
                write!(f, "There is no Pokemon with name:  \"{}\".", name)
            }
            ServiceError::TranslationFailed => write!(
                f,
                "It was not possible to translate the Pokemon description."
            ),
            ServiceError::ServiceUnavailable => write!(
                f,
                "The Pokemon description translation service is currently unavailable."
            ),
        }
    }
}

impl Error for ServiceError {}

impl Reply for PokemonData {
    fn into_response(self) -> Response {
        warp::reply::json(&self).into_response()
    }
}

impl Reject for ServiceError {}

impl Reply for ServiceError {
    fn into_response(self) -> Response {
        let msg = ErrorMessage::new(self.to_string());
        warp::reply::with_status(warp::reply::json(&msg), self.status_code()).into_response()
    }
}

async fn handle_rejection(rejection: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(e) = rejection.find::<ServiceError>() {
        Ok(e.clone())
    } else {
        Err(rejection)
    }
}
