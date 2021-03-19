use crate::model::PokemonData;
use futures::future::BoxFuture;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum PokemonServiceError {
    /// The service could not produce a result for any reason other than the record not existing.
    ServiceUnavailable,
    /// There is no Pokemon with the specified name available to the service.
    NoSuchPokemon(String),
}

impl Display for PokemonServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PokemonServiceError::ServiceUnavailable => {
                write!(f, "The Pokemon service is not currently available.",)
            }
            PokemonServiceError::NoSuchPokemon(name) => {
                write!(f, "There is no Pokemon with name:  \"{}\".", name)
            }
        }
    }
}

impl Error for PokemonServiceError {}

/// A service that can provide descriptions of Pokemon species given their name.
pub trait PokemonService {
    /// Attempt to get the description for a Pokemon with the specified species name.
    fn get_pokemon<'a>(
        &'a self,
        name: &'a str,
    ) -> BoxFuture<'a, Result<PokemonData, PokemonServiceError>>;
}
