use crate::model::PokemonData;
use crate::poke_api::model::{PokemonSpecies, FlavorEntry};
use crate::services::pokemon::{PokemonService, PokemonServiceError};
use futures::future::BoxFuture;
use futures::FutureExt;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{Client, Error, StatusCode, Url};

mod model;

lazy_static! {
    static ref WS: Regex = Regex::new(r"\s+").unwrap();
}

pub struct PokeApiService {
    client: Client,
    base_url: Url,
}

impl PokeApiService {
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
    fn get_pokemon(&self, name: &str) -> BoxFuture<Result<PokemonData, PokemonServiceError>> {
        let name = name.to_string();
        async move {
            let PokeApiService { client, .. } = self;
            let url = self.try_format_url(name.as_str())?;
            let response = client.get(url).send().await?;
            let status = response.status();

            if status.is_success() {
                let PokemonSpecies {
                    name,
                    flavor_text_entries
                } = response.json::<PokemonSpecies>().await?;
                if let Some(description) = select_description(flavor_text_entries) {
                    Ok(PokemonData { name, description })
                } else {
                    Err(PokemonServiceError::NoSuchPokemon(name))
                }
            } else if status == StatusCode::NOT_FOUND {
                Err(PokemonServiceError::NoSuchPokemon(name))
            } else {
                Err(PokemonServiceError::ServiceUnavailable)
            }
        }
        .boxed()
    }
}

const ENGLISH: &str = "en";

fn select_description(entries: Vec<FlavorEntry>) -> Option<String> {
    entries.into_iter().rev().find(|fl| fl.language.name == ENGLISH)
        .map(|FlavorEntry { flavor_text, .. }| clean_flavor_text(flavor_text))
}

fn clean_flavor_text(text: String) -> String {
    WS.replace_all(text.as_str(), " ").into()
}
