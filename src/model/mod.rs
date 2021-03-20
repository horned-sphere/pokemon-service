use serde::Serialize;

/// Model for the return type for our service endpoint.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PokemonData {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Debug)]
/// Model for error responses.
pub struct ErrorMessage {
    pub message: String,
}

impl ErrorMessage {
    pub fn new(message: String) -> Self {
        ErrorMessage { message }
    }
}
