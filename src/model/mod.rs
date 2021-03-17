use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PokemonData {
    name: String,
    description: String,
}