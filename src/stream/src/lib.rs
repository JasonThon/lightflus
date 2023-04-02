pub mod connector;
mod dataflow;
pub mod edge;
pub mod err;
mod state;
pub mod task;
mod v8_runtime;

pub type Receiver<Output> = tokio::sync::mpsc::Receiver<Output>;
pub type Sender<Output> = tokio::sync::mpsc::Sender<Output>;

#[cfg(feature = "v8_init")]
pub fn initialize_v8() {
    // v8::V8::set_flags_from_string(
    //     "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",
    //   );
    v8::V8::initialize_platform(v8::new_default_platform(10, false).make_shared());
    v8::V8::initialize();
}

pub fn new_event_channel<T>(buf_size: usize) -> (Sender<T>, Receiver<T>) {
    tokio::sync::mpsc::channel(buf_size)
}
