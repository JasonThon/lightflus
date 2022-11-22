FROM rust:1.61.0 as builder
WORKDIR /lightflus/runtime
COPY . .
RUN cargo install --path /lightflus/runtime/src/apiserver --target-dir /lightflus/runtime/target

FROM debian:buster-slim
WORKDIR /lightflus/runtime
COPY --from=builder /lightflus/runtime/src/apiserver/etc /lightflus/runtime
COPY --from=builder /lightflus/runtime/target/release/apiserver /lightflus/runtime/apiserver
CMD ["./apiserver", "-c", "etc/apiserver.json"]