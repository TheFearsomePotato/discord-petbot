FROM rust as builder
WORKDIR /usr/src/petbot
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/petbot/release/petbot ./petbot
COPY --from=builder /usr/src/petbot/assets ./assets
CMD ["./petbot"]