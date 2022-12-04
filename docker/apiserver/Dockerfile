FROM rust:1.62.0 as builder
WORKDIR /lightflus/runtime
COPY . .
# for local build
# COPY docker/cargo ${CARGO_HOME}/config
RUN cargo install --path /lightflus/runtime/src/apiserver --target-dir /lightflus/runtime/target

FROM debian:bullseye-slim
WORKDIR /lightflus/runtime
COPY --from=builder /lightflus/runtime/target/release/apiserver /lightflus/runtime/apiserver
EXPOSE 8080
CMD ["./apiserver"]