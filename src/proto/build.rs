/**
 * If proto has been changed. You must remove all comments and rerun build.rs to generate new rust files
 */
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut config = prost_build::Config::new();
    // config
    //     .bytes(&["common.Entry.value"])
    //     .type_attribute(
    //         "common.Entry",
    //         "#[derive(serde::Serialize,serde::Deserialize,Eq)]",
    //     )
    //     .type_attribute(
    //         "common.KeyedDataEvent",
    //         "#[derive(serde::Serialize,serde::Deserialize,Eq)]",
    //     )
    //     .type_attribute(
    //         "common.KeyedEventSet",
    //         "#[derive(serde::Serialize,serde::Deserialize,Eq)]",
    //     )
    //     .type_attribute(
    //         "common.KeyedDataEvent.Window",
    //         "#[derive(serde::Serialize,serde::Deserialize,Eq)]",
    //     )
    //     .type_attribute(
    //         "common.ResourceId",
    //         "#[derive(serde::Serialize,serde::Deserialize,Eq,PartialOrd,Ord,Hash)]",
    //     )
    //     .type_attribute(
    //         "common.HostAddr",
    //         "#[derive(serde::Serialize,serde::Deserialize,Eq,Hash)]",
    //     )
    //     .type_attribute(
    //         "common.SubDataflowId",
    //         "#[derive(serde::Serialize,serde::Deserialize,Eq,Hash,PartialOrd)]",
    //     )
    //     .out_dir("src");

    // let generator = tonic_build::configure().service_generator();
    // config.service_generator(generator).compile_protos(
    //     &[
    //         "../../proto/common/common.proto",
    //         "../../proto/common/event.proto",
    //         "../../proto/common/stream.proto",
    //         "../../proto/coordinator/coordinator.proto",
    //         "../../proto/taskmanager/taskmanager.proto",
    //         "../../proto/apiserver/apiserver.proto",
    //     ],
    //     &["../../proto"],
    // )?;
    Ok(())
}
