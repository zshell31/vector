use super::InternalEvent;
use metrics::counter;
use std::error::Error;
use std::fmt::Debug;

#[derive(Debug)]
pub struct WebSocketEventSendSuccess {
    pub byte_size: usize,
}

impl InternalEvent for WebSocketEventSendSuccess {
    fn emit_logs(&self) {
        trace!(message = "Processed one event.");
    }

    fn emit_metrics(&self) {
        counter!("processed_bytes_total", self.byte_size as u64);
    }
}

#[derive(Debug)]
pub struct WebSocketEventSendFail<E> {
    pub error: E,
}

impl<E> InternalEvent for WebSocketEventSendFail<E>
where
    E: Error + Debug,
{
    fn emit_logs(&self) {
        error!(message = "Failed to send message.", error = %self.error);
    }

    fn emit_metrics(&self) {
        counter!("send_errors_total", 1);
    }
}
