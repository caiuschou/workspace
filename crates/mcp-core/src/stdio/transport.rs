/// Shared interface for transports that exchange JSON-RPC messages.
pub trait Transport {
    /// The message type exchanged by the transport.
    type Message;

    /// The error type returned by transport operations.
    type Error;

    /// Acquire resources needed to talk to a remote endpoint.
    fn start(&mut self) -> Result<(), Self::Error>;

    /// Send a message to the remote endpoint.
    fn send(&mut self, message: &Self::Message) -> Result<(), Self::Error>;

    /// Close the transport and release its resources.
    fn close(&mut self) -> Result<(), Self::Error>;
}
