use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PokemonSpecies {
    name: String,
    flavor_text_entries: Vec<FlavorEntry>
}

#[derive(Deserialize, Debug)]
pub struct Language {
    name: String
}

#[derive(Deserialize, Debug)]
pub struct Version {
    name: String,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct FlavorEntry {
    flavor_text: String,
    language: Language,
    version: Version,
}

