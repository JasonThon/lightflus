FROM golang:1.17.7-alpine3.15 as tools-preparation
WORKDIR /lightflus/runtime/tools
COPY ./tools .
RUN go build -ldflags="-s -w" -o /lightflus/runtime/tools/healthcheck health-check/main.go

FROM rust:1.61.0-alpine as builder
WORKDIR /lightflus/runtime
COPY . .
RUN cargo install --path src/worker
COPY target/release/worker /lightflus/runtime/worker
COPY src/worker/etc /lightflus/runtime

FROM alpine:3.16
RUN apk add --update --no-cache rocksdb-dev
WORKDIR /lightflus/runtime
COPY --from=builder /lightflus/runtime/etc /lightflus/runtime/etc
COPY --from=builder /lightflus/runtime/worker /lightflus/runtime/worker
COPY --from=tools-preparation /lightflus/runtime/tools/healthcheck /lightflus/runtime/tools/healthcheck 
CMD ["./worker", "-c", "etc/worker.json"]