use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PokemonData {
    pub name: String,
    pub description: String,
}
