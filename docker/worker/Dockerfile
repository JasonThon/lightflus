FROM golang:1.17.7 as tools-preparation
WORKDIR /lightflus/runtime/tools
COPY ./tools .
RUN go build -ldflags="-s -w" -o /lightflus/runtime/tools/healthcheck health-check/main.go

FROM rust:1.62.0 as builder
WORKDIR /lightflus/runtime
COPY . .
# for local build
# COPY docker/cargo ${CARGO_HOME}/config
RUN cargo build --manifest-path /lightflus/runtime/src/worker/Cargo.toml --target-dir /lightflus/runtime/target

FROM debian:bullseye
WORKDIR /lightflus/runtime
RUN apt-get update && apt-get -y install make build-essential cmake curl pkg-config bash lld librdkafka-dev
COPY --from=tools-preparation /lightflus/runtime/tools/healthcheck /lightflus/tools/healthcheck 
COPY --from=builder /lightflus/runtime/target/release/worker /lightflus/runtime/worker 
COPY --from=builder /lightflus/runtime/src/worker/etc /lightflus/runtime/etc
EXPOSE 8792
CMD ["./worker", "-c", "etc/worker.json"]