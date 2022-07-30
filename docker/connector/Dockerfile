FROM rust:1.61.0 as builder
WORKDIR /tableflow/runtime
RUN apt-get update && apt-get install -y cmake

COPY . .
RUN cargo install --path src/connector

FROM alpine:3.15
WORKDIR /tableflow/runtime
COPY --from=builder /tableflow/runtime/src/connector/etc /tableflow/runtime
COPY --from=builder /tableflow/runtime/target/release/connector /tableflow/runtime/connector
CMD ["./connector", "-f", "etc/connector"]