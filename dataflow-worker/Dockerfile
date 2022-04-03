FROM rust:1.59-alpine as builder
WORKDIR /opt/tableflow/dataflow/worker
COPY . .
RUN cargo install --path .

FROM alpine:3.14
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /opt/tableflow/dataflow/worker /opt/dataflow/worker
CMD ["myapp"]