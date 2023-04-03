FROM golang:1.17.7 as tools-preparation
WORKDIR /lightflus/runtime/tools
COPY ./tools .
RUN go build -ldflags="-s -w" -o /lightflus/runtime/tools/healthcheck health-check/main.go

FROM rust:1.68.0 as builder
WORKDIR /lightflus/runtime
RUN apt-get update && apy-get upgrade -y && apt-get -y install make build-essential cmake curl pkg-config bash lld
COPY docker/config ${CARGO_HOME}/config
COPY . .
RUN cargo build --release --manifest-path /lightflus/runtime/src/taskmanager/Cargo.toml --target-dir /lightflus/runtime/target

FROM debian:bullseye
WORKDIR /lightflus/runtime
COPY --from=tools-preparation /lightflus/runtime/tools/healthcheck /lightflus/runtime/tools/healthcheck 
COPY --from=builder /lightflus/runtime/target/release/taskmanager /lightflus/runtime/taskmanager 
COPY --from=builder /lightflus/runtime/src/taskmanager/etc /lightflus/runtime/etc
EXPOSE 8792
CMD ["./taskmanager", "-c", "etc/taskmanager.json"]