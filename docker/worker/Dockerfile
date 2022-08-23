FROM golang:1.17.7-alpine3.15 as tools-preparation
WORKDIR /tableflow/runtime/tools
COPY ./tools .
RUN go build -ldflags="-s -w" -o /tableflow/runtime/tools/healthcheck health-check/main.go

FROM rust:1.61.0-alpine as builder
WORKDIR /tableflow/runtime
COPY . .
RUN cargo install --path src/worker
COPY target/release/worker /tableflow/runtime/worker
COPY src/worker/etc /tableflow/runtime

FROM alpine:3.15
WORKDIR /tableflow/runtime
COPY --from=builder /tableflow/runtime/etc /tableflow/runtime/etc
COPY --from=builder /tableflow/runtime/worker /tableflow/runtime/worker
COPY --from=tools-preparation /tableflow/runtime/tools/healthcheck /tableflow/runtime/tools/healthcheck 
CMD ["./worker", "-c", "etc/worker.json"]