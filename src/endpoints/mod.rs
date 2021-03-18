use crate::model::PokemonData;
use crate::services::pokemon::{PokemonService, PokemonServiceError};
use crate::services::translation::{TranslationError, TranslationService};
use reqwest::StatusCode;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use warp::reject::Reject;
use warp::{Filter, Rejection, Reply};

pub async fn run_server<Poke, Trans>(
    socket_addr: SocketAddr,
    pokemon_service: Poke,
    translation_service: Trans,
) where
    Poke: PokemonService + Send + Sync + 'static,
    Trans: TranslationService + Send + Sync + 'static,
{
    let shared_pokemon_service = Arc::new(pokemon_service);
    let shared_translation_service = Arc::new(translation_service);

    let pokemon_service_filter = warp::any().map(move || shared_pokemon_service.clone());
    let shared_translation_service = warp::any().map(move || shared_translation_service.clone());

    let endpoint = warp::path!("pokemon" / String)
        .and(pokemon_service_filter)
        .and(shared_translation_service)
        .and_then(|name, pokemon, trans| async move {
            handle_request(name, pokemon, trans)
                .await
                .map_err(warp::reject::custom)
        })
        .map(|data: PokemonData| warp::reply::json(&data))
        .recover(handle_rejection);

    warp::serve(endpoint).run(socket_addr).await
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

#[derive(Debug)]
enum ServiceError {
    NoSuchPokemon(String),
    TranslationFailed,
    ServiceUnavailable,
}

impl Reject for ServiceError {}

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

async fn handle_rejection(rejection: Rejection) -> Result<impl Reply, Rejection> {
    match rejection.find::<ServiceError>() {
        Some(e @ ServiceError::NoSuchPokemon(_)) => Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::NOT_FOUND,
        )),
        Some(e @ ServiceError::TranslationFailed) => Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
        Some(e @ ServiceError::ServiceUnavailable) => Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::SERVICE_UNAVAILABLE,
        )),
        _ => Err(rejection),
    }
}
