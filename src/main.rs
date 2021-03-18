mod endpoints;
pub mod model;
mod poke_api;
pub mod services;
mod shakespeare_api;

use crate::poke_api::PokeApiService;
use crate::shakespeare_api::ShakespeareService;
use clap::Clap;
use reqwest::Url;
use std::net::{IpAddr, SocketAddr};

#[derive(Clap)]
#[clap(name = "pokemon-translator", version = "0.1")]
struct Params {
    /// Address to bind to
    #[clap(short, long, default_value = "127.0.0.1")]
    bind: String,
    /// Port to bind to
    #[clap(short, long)]
    port: u16,
    /// Base URL of the Pokemon API
    #[clap(short = 'a', long)]
    pokemon: String,
    /// URL of the Shakespeare translation service
    #[clap(short, long)]
    shakespeare: String,
}

impl Params {
    fn validate(self) -> Result<(SocketAddr, Url, Url), String> {
        let Params {
            bind,
            port,
            pokemon,
            shakespeare,
        } = self;

        let addr: IpAddr = bind
            .parse()
            .map_err(|_| format!("{} is not a valid IP.", bind))?;

        let sock_addr = SocketAddr::new(addr, port);

        let poke_api_url =
            Url::parse(pokemon.as_str()).map_err(|_| format!("{} is not a valid URL.", pokemon))?;

        let shakespeare_url = Url::parse(shakespeare.as_str())
            .map_err(|_| format!("{} is not a valid URL.", shakespeare))?;

        Ok((sock_addr, poke_api_url, shakespeare_url))
    }
}

#[tokio::main]
async fn main() {
    let params: Params = Params::parse();

    match params.validate() {
        Ok((sock_addr, poke_api_url, shakespeare_url)) => {
            let client = reqwest::Client::new();

            let pokemon_service = PokeApiService::new(client.clone(), poke_api_url);
            let shakespeare_service = ShakespeareService::new(client, shakespeare_url);
            endpoints::run_server(sock_addr, pokemon_service, shakespeare_service).await;
        }
        Err(msg) => {
            panic!(msg);
        }
    }
}
