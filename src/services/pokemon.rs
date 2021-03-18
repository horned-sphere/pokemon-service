use futures::future::BoxFuture;
use crate::model::PokemonData;
use std::fmt::{Display, Formatter};
use std::error::Error;

#[derive(Debug)]
pub enum PokemonServiceError {
    ServiceUnavailable,
    NoSuchPokemon(String)
}

impl Display for PokemonServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PokemonServiceError::ServiceUnavailable => write!(f, "The Pokemon service is not currently available.", ),
            PokemonServiceError::NoSuchPokemon(name) => write!(f, "There is no Pokemon with name:  \"{}\".", name),
        }
    }
}

impl Error for PokemonServiceError {}

pub trait PokemonService {

    fn get_pokemon<'a>(&'a self, name: &'a str) -> BoxFuture<'a, Result<PokemonData, PokemonServiceError>>;

}