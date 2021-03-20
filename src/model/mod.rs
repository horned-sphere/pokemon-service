use serde::Serialize;

/// Model for the return type for our service endpoint.
#[derive(Serialize, Debug)]
pub struct PokemonData {
    pub name: String,
    pub description: String,
}
