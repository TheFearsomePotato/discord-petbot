FROM rust as builder
WORKDIR /appsrc
RUN cargo new --bin discord-petbot
WORKDIR /appsrc/discord-petbot

COPY ./Cargo.toml .
RUN cargo build --release
RUN rm -r ./src
RUN rm ./target/release/discord-petbot
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /appsrc/discord-petbot/target/release/discord-petbot ./discord-petbot
COPY ./assets ./assets
CMD ["./discord-petbot"]