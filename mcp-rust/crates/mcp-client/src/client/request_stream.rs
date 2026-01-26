use std::sync::mpsc::Receiver;

use crate::client::ResponseMessage;

/// Blocking stream of responses for a request.
#[derive(Debug)]
pub struct RequestStream {
    receiver: Receiver<ResponseMessage>,
}

impl RequestStream {
    pub(crate) fn new(receiver: Receiver<ResponseMessage>) -> Self {
        Self { receiver }
    }

    pub fn recv(&self) -> Option<ResponseMessage> {
        self.receiver.recv().ok()
    }
}

impl Iterator for RequestStream {
    type Item = ResponseMessage;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok()
    }
}
