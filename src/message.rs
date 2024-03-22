//! Provides a type representing a TWS protocol message as well as utilities for
//! parsing messages from a byte array.

use bytes::{Buf, Bytes};
use std::convert::TryInto;
use std::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

/// A message in the TWS protocol.
#[derive(Clone, Debug)]
pub enum Message {
    Handshake(Bytes),
    Bulk(Bytes),
    Null,
}

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(crate::Error),
}

impl Message {
    /// Checks if an entire message can be decoded from `src`
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        // Most likely I can simplify that by checking message length > 4
        match get_prefix(src)? {
            b"API" => {
                get_line(src)?;
                Ok(())
            }
            _ => {
                get_decimal(src)?;
                Ok(())
            } // actual => {
              //     Err(format!("protocol error; invalid message type byte `{:?}`", actual).into())
              // }
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Message, Error> {
        println!("inside parse");
        match get_prefix(src)? {
            b"API" => {
                println!("Matched API inside parse");
                // Read the line and convert it to `Vec<u8>`
                let line = get_line(src)?.to_vec();
                Ok(Message::Handshake(line.into()))
            }
            _ => unimplemented!(),
        }
    }

    /// Converts the message to an "unexpected message" error
    pub(crate) fn to_error(&self) -> crate::Error {
        format!("unexpected message: {}", self).into()
    }
}

impl PartialEq<&str> for Message {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Message::Bulk(s) => s.eq(other),
            _ => false,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Message::Handshake(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Message::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Message::Null => "(nil)".fmt(fmt),
        }
    }
}

// Reads first 4 bytes and returns a message prefix which
// can be a message size or "API" in case of negotiation
fn get_prefix<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    return Ok(&src.get_ref()[0..3]);
}

/// Read a new-line terminated decimal
fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(src)?;

    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

/// Find a line
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let end = src.get_ref().len();

    if end > 4 {
        return Ok(&src.get_ref()[..end]);
    }

    Err(Error::Incomplete)
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}
