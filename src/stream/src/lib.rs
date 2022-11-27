pub mod actor;
pub mod dataflow;
pub mod err;
pub mod state;
pub mod v8_runtime;

pub(crate) static MOD_TEST_START: std::sync::Once = std::sync::Once::new();

pub(crate) type EventReceiver<Input> = crossbeam_channel::Receiver<Input>;
pub(crate) type EventSender<Input> = crossbeam_channel::Sender<Input>;
pub(crate) const DETAULT_WATERMARK: std::time::Duration = std::time::Duration::from_millis(100);

pub fn initialize_v8() {
    // v8::V8::set_flags_from_string(
    //     "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",
    //   );
    v8::V8::initialize_platform(v8::new_default_platform(10, false).make_shared());
    v8::V8::initialize();
}
