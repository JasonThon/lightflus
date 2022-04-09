FROM alpine:3.14
WORKDIR /dataflow
COPY ../../target/release/dataflow-worker /dataflow/worker
CMD ["./worker"]