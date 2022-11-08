FROM rust:1.61.0-alpine as builder
WORKDIR /lightflus/runtime
COPY . .
RUN cargo install --path src/apiserver
COPY target/release/apiserver /lightflus/runtime/apiserver
COPY src/apiserver/etc /lightflus/runtime

FROM alpine:3.15
RUN apk add --update --no-cache rocksdb-dev
WORKDIR /lightflus/runtime
COPY --from=builder /lightflus/runtime/etc /lightflus/runtime/etc
COPY --from=builder /lightflus/runtime/apiserver /lightflus/runtime/apiserver
CMD ["./apiserver", "-c", "etc/apiserver.json"]