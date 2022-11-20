pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().out_dir("src").compile(
        &[
            "../../proto/common/common.proto",
            "../../proto/common/event.proto",
            "../../proto/common/probe.proto",
            "../../proto/common/stream.proto",
        ],
        &["../../proto"],
    )?;

    tonic_build::configure().out_dir("src").compile(
        &[
            "../../proto/coordinator/coordinator.proto",
            "../../proto/worker/worker.proto",
            "../../proto/apiserver/apiserver.proto",
        ],
        &["../../proto"],
    )?;

    Ok(())
}
