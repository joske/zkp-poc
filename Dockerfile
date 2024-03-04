FROM rust:latest as builder

RUN apt update && apt install -y protobuf-compiler

WORKDIR /usr/src/zkp-poc

COPY . .

RUN cargo build --release --bin server --bin client

FROM debian:latest as server
COPY --from=builder /usr/src/zkp-poc/target/release/server /usr/local/bin/server
ENTRYPOINT ["/usr/local/bin/server"]

FROM debian:latest as client
COPY --from=builder /usr/src/zkp-poc/target/release/client /usr/local/bin/client
ENTRYPOINT ["/usr/local/bin/client"]
