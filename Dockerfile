FROM rust as builder
WORKDIR /appsrc
RUN cargo new --bin petbot
WORKDIR /appsrc/petbot

COPY ./Cargo.toml .
RUN cargo build --release
RUN rm -r ./src
COPY ./src /appsrc/petbot/src
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/petbot/release/petbot ./petbot
COPY ./assets ./assets
CMD ["./petbot"]