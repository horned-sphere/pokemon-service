use crate::poke_api::model::{FlavorEntry, Language, PokemonSpecies, Version};
use crate::poke_api::PokeApiService;
use reqwest::{Client, Url};

const SAMPLE: &str = include_str!("sample.json");

const RAW_DESC: &str = "When several of\nthese POKéMON\ngather, their\x0celectricity could\nbuild and cause\nlightning storms.";
const EXPECTED_DESC: &str = "When several of these POKéMON gather, their electricity could build and cause lightning storms.";

#[test]
fn deserialize_service_response() {
    let result = serde_json::from_str::<PokemonSpecies>(SAMPLE);
    assert!(result.is_ok());

    let PokemonSpecies {
        name,
        flavor_text_entries,
    } = result.unwrap();

    assert_eq!(name, "pikachu");
    assert_eq!(flavor_text_entries.len(), 328);
}

#[test]
fn pick_description() {
    let descriptions = vec![
        make_flavor("en", "Description 1", "a"),
        make_flavor("de", "Description 2", "b"),
        make_flavor("en", "Description 3", "a"),
        make_flavor("en", "Description 4", "c"),
        make_flavor("fr", "Description 5", "c"),
    ];

    let selected = super::select_description(descriptions);

    assert_eq!(selected, Some("Description 4".to_string()));
}

fn make_flavor(language: &str, description: &str, ver: &str) -> FlavorEntry {
    FlavorEntry {
        flavor_text: description.to_string(),
        language: Language {
            name: language.to_string(),
        },
        version: Version {
            name: ver.to_string(),
        },
    }
}

#[test]
fn clean_description() {
    let cleaned = super::clean_flavor_text(RAW_DESC);
    assert_eq!(cleaned, EXPECTED_DESC);
}

const SERVICE_URL: &str = "https://pokeapi.co/api/v2/pokemon-species";

#[test]
fn format_url() {
    let url = Url::parse(SERVICE_URL).unwrap();

    let service = PokeApiService::new(Client::new(), url);

    let result = service.try_format_url("pikachu").map(|u| u.to_string());

    assert_eq!(
        result,
        Ok("https://pokeapi.co/api/v2/pokemon-species/pikachu".to_string())
    );
}

#[test]
fn check_names() {
    assert!(super::NAME.is_match("pikachu"));
    assert!(super::NAME.is_match("Pikachu"));
    assert!(super::NAME.is_match("two-part"));
    assert!(super::NAME.is_match("pikachu2"));
}

#[cfg(feature = "api_tests")]
#[tokio::test]
async fn call_service() {
    use crate::model::PokemonData;
    use crate::services::pokemon::PokemonService;

    let url = Url::parse(SERVICE_URL).unwrap();

    let service = PokeApiService::new(Client::new(), url);
    let result = service.get_pokemon("pikachu").await;

    match result {
        Ok(PokemonData { name, description }) => {
            assert_eq!(name, "pikachu");
            assert_eq!(super::clean_flavor_text(description.as_str()), description);
        }
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}
