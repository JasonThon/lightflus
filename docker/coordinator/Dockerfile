FROM rust:1.61.0-alpine as builder
WORKDIR /tableflow/runtime
COPY . .
RUN cargo install --path src/coordinator
COPY target/release/coordinator /tableflow/runtime/coordinator
COPY src/coordinator/etc /tableflow/runtime

FROM alpine:3.15
RUN apk add --update --no-cache rocksdb-dev
WORKDIR /tableflow/runtime
COPY --from=builder /tableflow/runtime/etc /tableflow/runtime/etc
COPY --from=builder /tableflow/runtime/coordinator /tableflow/runtime/coordinator
CMD ["./coordinator", "-c", "./etc/coordinator.json"]