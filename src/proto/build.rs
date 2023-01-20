/**
 * If proto has been changed. You must remove all comments and rerun build.rs to generate new rust files
 */
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::configure().out_dir("src").compile(
    //     &[
    //         "../../proto/common/common.proto",
    //         "../../proto/common/event.proto",
    //         "../../proto/common/stream.proto",
    //     ],
    //     &["../../proto"],
    // )?;

    // tonic_build::configure().out_dir("src").compile(
    //     &[
    //         "../../proto/coordinator/coordinator.proto",
    //         "../../proto/worker/worker.proto",
    //         "../../proto/apiserver/apiserver.proto",
    //     ],
    //     &["../../proto"],
    // )?;

    Ok(())
}
