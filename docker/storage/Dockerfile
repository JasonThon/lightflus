FROM golang:1.17.7-alpine3.15 as tools-preparation
WORKDIR /tableflow/runtime/tools
COPY ./tools .
RUN go build -ldflags="-s -w" -o /tableflow/runtime/tools/healthcheck health-check/main.go

FROM rust:1.61.0-alpine as builder
WORKDIR /tableflow/runtime
COPY . .
RUN cargo install --path src/coordinator
COPY target/release/storage /tableflow/runtime/storage
COPY src/storage/etc /tableflow/runtime

FROM alpine:3.15
WORKDIR /tableflow/runtime
COPY --from=builder /tableflow/runtime/etc /tableflow/runtime/etc
COPY --from=builder /tableflow/runtime/storage /tableflow/runtime/storage
COPY --from=tools-preparation /tableflow/runtime/tools/healthcheck /tableflow/runtime/tools/healthcheck 
CMD ["./storage", "-c", "etc/storage.json"]