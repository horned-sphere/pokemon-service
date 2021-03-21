Shakespearian Pokemon Translation Service
========================================

Description
-----------

Web service that fetches the description of a Pokemon from the PokeAPI service (https://pokeapi.co) and converts it into faux Shakespearian English (using https://funtranslations.com/api/#shakespeare).

The service can be called as:

`curl http://localhost:8080/pokemon/{name}`

and will return a JSON document of the form:

`{ name: "...", description: "..." }`

Building
--------

The service will work with Rust 1.50 and can be built using cargo:

`cargo build --release`

For a complete description of the parameters:

`cargo run --release -- --help`

To run the service (binding by default to 127.0.0.1:8080):

`cargo run --release -- --pokemon https://pokeapi.co/api/v2/pokemon-species --shakespeare https://api.funtranslations.com/translate/shakespeare.json`

Logging can be enabled with the `RUST_LOG` environment variable.

Building with Docker
--------------------

A Dockerfile is provided to build an image for the service. This can be built with:

`docker build -t pokeservice .`

To run the service (the default of binding to 127.0.0.1 will not work inside a Docker container):

`docker run -p 8080:8080 pokeservice --bind 0.0.0.0 --pokemon https://pokeapi.co/api/v2/pokemon-species --shakespeare https://api.funtranslations.com/translate/shakespeare.json`

Testing
-------

The service has unit tests that can be run with.

`cargo test`

A few integration test cases call the real services and are disabled by default. They can be enabled with:

`cargo tests --features api_tests`

These should be avoided in general due to rate limiting and ideally would be removed entirely (see improvements below).

Potential Improvements
----------------------

There are a number of ways in which the service could be improved.

* The service uses warp (as an HTTP server) and reqwest (for making requests to the delegate services). Both of these are used with default configuration. It would be better to expose the configuration in the application with a configuration file.
* Currently, Pokemon descriptions are fetched by species name. Some species have a number of sub-variants which will not be found by the current implementation.
* Pokemon will only be found if the name used by the PokeAPI service, for the species, on the API endpoint is used. For example 'Mr. Mime' must be referred to as 'mr-mime'. A better implementation would be able to resolve different forms of the name.
* The PokeAPI service returns many alternative descriptions for each species, from different versions of the game and in different languages. Currently, we chose the last (in the returned JSON array) description that is in English. It would be better to either explicitly chose the description from the latest _version_ or to have a configurable preferred version.
* The PokeAPI keeps the line breaks and form feeds from the original game text which this service strips out for readability. I the vast majority of cases these characters can safely be replaced with a space. However, in some descriptions this causes spurious spaces to be inserted (for example around hyphenation across line breaks). This could be improved to avoid this.
* The service does not currently support TLS.
* The Shakespeare translation API has a paid version with an API key. They configuration for the service could be extended to allow a key to be supplied.
* The interfaces for the Pokemon and translation services are defined as traits. Due to current compiler limitations around associated types this necessitates boxing the futures. This could be avoided by encoding the interfaces using function traits (for example, the translation API could be defined as `Fn(&'a str) -> Fut, Fut: Future<Output = String> + 'a`). This would (potentially) improve performance a tht expense of some readability.
* The reqwest API is not easy to mock. As a quick solution I have written some integration tests that run directly against the real APIs. It would be better to design an abstraction around the reqwest client or to spin up a minimal server in the test cases, however, this would have been very time-consuming.
* Building the docker image will always rebuild all the dependencies. This can be avoided but the solutions I have seen all look quite hacky so I didn't use any of them.