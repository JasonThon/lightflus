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
CMD ["./worker", "-c", "/tableflow/runtime/etc/worker.json"]