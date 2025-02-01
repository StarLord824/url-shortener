FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates

COPY --from=builder /app/target/release/creative-url-shortener /app/creative-url-shortener
COPY .env /app/

WORKDIR /app
CMD ["/app/creative-url-shortener"]