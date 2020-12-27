FROM rust:1.47

RUN apt-get update && apt-get install -y cmake clang  \
    && rm -rf /var/lib/apt/lists/*

WORKDIR app
COPY . .
RUN cargo build
