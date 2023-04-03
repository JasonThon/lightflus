pub mod env_keys {
    pub const CHANNEL_SIZE: &str = "lightflus.operator.channel.size";
    pub const SEND_OPERATOR_EVENT_CONNECT_TIMEOUT: &str =
        "lightflus.send_operator_event.connect_timeout";
    pub const SEND_OPERATOR_EVENT_RPC_TIMEOUT: &str = "lightflus.send_operator_event.rpc_timeout";
}

pub mod default_configs {
    pub const DEFAULT_CHANNEL_SIZE: usize = 1000;
    pub const DEFAULT_SEND_OPERATOR_EVENT_RPC_TIMEOUT_MILLIS: u64 = 3000;
    pub const DEFAULT_SEND_OPERATOR_EVENT_CONNECT_TIMEOUT_MILLIS: u64 = 3000;
}
