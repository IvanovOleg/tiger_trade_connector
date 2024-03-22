use crate::{Connection, Message, Parse, ParseError};
use bytes::Bytes;
use tracing::{debug, instrument};

/// Returns PONG if no argument is provided, otherwise
/// return a copy of the argument as a bulk.
///
/// This command is often used to test if a connection
/// is still alive, or to measure latency.
#[derive(Debug, Default)]
pub struct StartApi {
    /// optional message to be returned
    msg: Option<Bytes>,
}

impl StartApi {
    /// Create a new `Ping` command with optional `msg`.
    pub fn new(msg: Option<Bytes>) -> StartApi {
        StartApi { msg }
    }

    /// Parse a `Ping` instance from a received frame.
    ///
    /// The `Parse` argument provides a cursor-like API to read fields from the
    /// `Frame`. At this point, the entire frame has already been received from
    /// the socket.
    ///
    /// The `PING` string has already been consumed.
    ///
    /// # Returns
    ///
    /// Returns the `Ping` value on success. If the frame is malformed, `Err` is
    /// returned.
    ///
    /// # Format
    ///
    /// Expects an array frame containing `PING` and an optional message.
    ///
    /// ```text
    /// PING [message]
    /// ```
    pub(crate) fn parse_message(parse: &mut Parse) -> crate::Result<StartApi> {
        match parse.next_bytes() {
            Ok(msg) => Ok(StartApi::new(Some(msg))),
            Err(ParseError::EndOfStream) => Ok(StartApi::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Apply the `Ping` command and return the message.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let response = match self.msg {
            None => Message::Bulk("PONG".into()), // should send b'\x00\x00\x00\x1a176\x0020240209 22:23:12 EST\x00'
            Some(msg) => Message::Bulk(msg),
        };

        debug!(?response);

        // Write the response back to the client
        dst.write_message(&response).await?;

        Ok(())
    }
}
