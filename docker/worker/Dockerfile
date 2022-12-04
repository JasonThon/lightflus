# FROM golang:1.17.7 as tools-preparation
# WORKDIR /lightflus/runtime/tools
# COPY ./tools .
# RUN go build -ldflags="-s -w" -o /lightflus/runtime/tools/healthcheck health-check/main.go

# FROM rust:1.62.0 as builder
# WORKDIR /lightflus/runtime
# COPY . .
# for local build
# COPY docker/cargo ${CARGO_HOME}/config
# RUN cargo build --target-dir /lightflus/runtime/src/worker --target-dir /lightflus/runtime/target

FROM debian:bullseye

WORKDIR /lightflus
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get -y install make build-essential cmake curl pkg-config bash lld
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --no-modify-path --default-toolchain none -y
COPY . /lightflus/runtime

# for local build
# COPY docker/cargo ${CARGO_HOME}/config

RUN cargo build --manifest-path /lightflus/runtime/src/worker --target-dir /lightflus/runtime/target --release
RUN cp /lightflus/runtime/target/release/worker /lightflus/worker

RUN cp /lightflus/runtime/src/worker/etc /lightflus/etc

RUN rm -rf /lightflus/runtime
# COPY --from=tools-preparation /lightflus/runtime/tools/healthcheck /lightflus/tools/healthcheck 
EXPOSE 8792
CMD ["./worker", "-c", "etc/worker.json"]