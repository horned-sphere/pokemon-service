use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PokemonSpecies {
    pub name: String,
    pub flavor_text_entries: Vec<FlavorEntry>,
}

#[derive(Deserialize, Debug)]
pub struct Language {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    pub name: String,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct FlavorEntry {
    pub flavor_text: String,
    pub language: Language,
    pub version: Version,
}
