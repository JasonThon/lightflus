use tokio::time;

pub enum Window {
    Sliding {
        size: time::Duration,
        period: time::Duration
    }
}