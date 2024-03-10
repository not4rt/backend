FROM rust:1.76.0 as base

RUN apt-get update -yqq

WORKDIR /app

FROM base as build

RUN mkdir src; touch src/main.rs

COPY Cargo.toml Cargo.lock ./
COPY src ./src/
COPY sql ./sql/

RUN cargo build --release

FROM base

COPY --from=build /app /app

EXPOSE 80

CMD ./target/release/backend