use crate::model::PokemonData;
use crate::poke_api::model::{FlavorEntry, PokemonSpecies};
use crate::services::pokemon::{PokemonService, PokemonServiceError};
use futures::future::BoxFuture;
use futures::FutureExt;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{Client, Error, StatusCode, Url};
use tracing::{event, Level};

mod model;
#[cfg(test)]
mod tests;

lazy_static! {
    static ref WS: Regex = Regex::new(r"\s+").unwrap();
    static ref NAME: Regex = Regex::new(r"^[A-Za-z][A-Za-z0-9\-]*$").unwrap();
}

/// Implementation of the Pokemon service that queries the PokeAPI at https://pokeapi.co.
pub struct PokeApiService {
    client: Client,
    base_url: Url,
}

impl PokeApiService {
    /// # Arguments
    /// * `client` - HTTP client to make remote requests.
    /// * `base-url` - Base url to the service. The name of the Pokemon will be appended as the
    /// final path segment.
    pub fn new(client: Client, base_url: Url) -> Self {
        PokeApiService { client, base_url }
    }
}

impl PokeApiService {
    fn try_format_url(&self, name: &str) -> Result<Url, PokemonServiceError> {
        let mut url = self.base_url.clone();
        if let Ok(mut path) = url.path_segments_mut() {
            path.push(name);
        } else {
            return Err(PokemonServiceError::ServiceUnavailable);
        }
        Ok(url)
    }
}

impl From<reqwest::Error> for PokemonServiceError {
    fn from(_: Error) -> Self {
        PokemonServiceError::ServiceUnavailable
    }
}

impl PokemonService for PokeApiService {
    fn get_pokemon<'a>(
        &'a self,
        name: &'a str,
    ) -> BoxFuture<'a, Result<PokemonData, PokemonServiceError>> {
        async move {
            // Check that the name is reasonable.
            if !NAME.is_match(name) {
                event!(Level::INFO, message = "Rejected ill-formed Pokemon name.", %name);
                return Err(PokemonServiceError::NoSuchPokemon(name.to_string()));
            }

            let PokeApiService { client, .. } = self;
            let url = self.try_format_url(name)?;
            event!(Level::DEBUG, message = "Making Pokemon species request to:", %url);
            let response = client.get(url).send().await?;
            let status = response.status();

            event!(Level::DEBUG, message = "Received response from Pokemon service.", %status);

            if status.is_success() {
                let PokemonSpecies {
                    name,
                    flavor_text_entries,
                } = response.json::<PokemonSpecies>().await?;
                if let Some(description) = select_description(flavor_text_entries) {
                    Ok(PokemonData { name, description })
                } else {
                    event!(Level::WARN, message = "No suitable description was available.", %name);
                    Err(PokemonServiceError::NoSuchPokemon(name))
                }
            } else if status == StatusCode::NOT_FOUND {
                Err(PokemonServiceError::NoSuchPokemon(name.to_string()))
            } else {
                event!(Level::ERROR, message = "Unanticipated response from Pokemon service.", %status);
                Err(PokemonServiceError::ServiceUnavailable)
            }
        }
        .boxed()
    }
}

/// Currently, we are only considering English descriptions.
const ENGLISH: &str = "en";

fn select_description(entries: Vec<FlavorEntry>) -> Option<String> {
    // Chooses the last description that is in English.
    entries
        .into_iter()
        .rev()
        .find(|fl| fl.language.name == ENGLISH)
        .map(|FlavorEntry { flavor_text, .. }| clean_flavor_text(flavor_text.as_str()))
}

fn clean_flavor_text(text: &str) -> String {
    // The descriptions returned by the service contain the original new-lines and line-feeds that
    // we don't want.
    WS.replace_all(text, " ").into()
}
