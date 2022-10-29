FROM golang:1.17.7-alpine3.15 as tools-preparation
WORKDIR /lightflus/runtime/tools
COPY ./tools .
RUN go build -ldflags="-s -w" -o /lightflus/runtime/tools/healthcheck health-check/main.go

FROM rust:1.61.0-alpine as builder
WORKDIR /lightflus/runtime
COPY . .
RUN cargo install --path src/coordinator
COPY target/release/coordinator /lightflus/runtime/coordinator
COPY src/coordinator/etc /lightflus/runtime

FROM alpine:3.15
RUN apk add --update --no-cache rocksdb-dev
WORKDIR /lightflus/runtime
COPY --from=builder /lightflus/runtime/etc /lightflus/runtime/etc
COPY --from=builder /lightflus/runtime/coordinator /lightflus/runtime/coordinator
COPY --from=tools-preparation /lightflus/runtime/tools/healthcheck /lightflus/runtime/tools/healthcheck 
CMD ["./coordinator", "-c", "etc/coordinator.json"]