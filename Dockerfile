FROM rust:1.50 as cargo-build

WORKDIR /tmp/pokeservice
COPY Cargo.lock .
COPY Cargo.toml .
COPY ./src src

RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/local/pokeservice

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata

ENV TZ=Etc/UTC \
    APP_USER=wsuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=cargo-build /tmp/pokeservice/target/release/pokeservice ${APP}/pokeservice

USER $APP_USER
WORKDIR ${APP}

ENTRYPOINT ["./pokeservice"]