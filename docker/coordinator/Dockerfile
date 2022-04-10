FROM alpine:3.14
WORKDIR /dataflow
COPY target/release/tableflow-rs /dataflow/coordinator
CMD ["./coordinator"]