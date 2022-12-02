FROM golang:1.17.7 as tools-preparation
WORKDIR /lightflus/runtime/tools
COPY ./tools .
RUN go build -ldflags="-s -w" -o /lightflus/runtime/tools/healthcheck health-check/main.go

FROM rust:1.61.0 as builder
WORKDIR /lightflus/runtime
COPY . .
RUN cargo install --path /lightflus/runtime/src/coordinator --target-dir /lightflus/runtime/target

FROM debian:buster-slim
WORKDIR /lightflus/runtime
COPY --from=builder /lightflus/runtime/src/coordinator/etc /lightflus/runtime
COPY --from=builder /lightflus/runtime/target/release/coordinator /lightflus/runtime/coordinator
COPY --from=tools-preparation /lightflus/runtime/tools/healthcheck /lightflus/runtime/tools/healthcheck 
CMD ["./coordinator", "-c", "etc/coordinator.json"]