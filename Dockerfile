FROM rust:1.85 AS builder
WORKDIR /opt
COPY . .
RUN cargo build --release
RUN cp /opt/target/release/watchlist-backend .
RUN cargo clean

FROM ubuntu:24.04
RUN apt-get update \
    && apt-get install -y --no-install-recommends vim \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /opt
COPY --from=builder /opt/watchlist-backend .
EXPOSE 8080
CMD ["./watchlist-backend"]