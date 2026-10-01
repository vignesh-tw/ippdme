//! A [`tokio_util::codec`] `Decoder`/`Encoder` that frames the I++ DME
//! CRLF-terminated ASCII protocol into parsed [`Message`] values.

use bytes::{BufMut, BytesMut};
use ippdme_core::{parse_message, Message};
use tokio_util::codec::{Decoder, Encoder};

use crate::error::NetError;

#[derive(Debug, Default)]
pub struct MessageCodec;

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = NetError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Message>, NetError> {
        // Find a line terminator, accepting either bare LF or CRLF.
        let Some(newline_pos) = src.iter().position(|&b| b == b'\n') else {
            return Ok(None);
        };

        let line = src.split_to(newline_pos + 1);
        let line = &line[..line.len() - 1]; // drop the '\n'
        let line = if line.last() == Some(&b'\r') {
            &line[..line.len() - 1]
        } else {
            line
        };

        if line.is_empty() {
            return self.decode(src);
        }

        let text = std::str::from_utf8(line)
            .map_err(|e| NetError::Protocol(ippdme_core::IppError::Parse(e.to_string())))?;
        let message = parse_message(text)?;
        Ok(Some(message))
    }
}

/// What a client writes to the wire: a typed message, or a line of text sent
/// verbatim (for hand-typed or deliberately malformed input).
#[derive(Debug, Clone, PartialEq)]
pub enum Outgoing {
    Message(Message),
    RawLine(String),
}

impl From<Message> for Outgoing {
    fn from(msg: Message) -> Self {
        Outgoing::Message(msg)
    }
}

impl Encoder<Outgoing> for MessageCodec {
    type Error = NetError;

    fn encode(&mut self, item: Outgoing, dst: &mut BytesMut) -> Result<(), NetError> {
        match item {
            Outgoing::Message(msg) => self.encode(msg, dst),
            Outgoing::RawLine(line) => write_line(line, dst),
        }
    }
}

impl Encoder<Message> for MessageCodec {
    type Error = NetError;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), NetError> {
        write_line(item.to_string(), dst)
    }
}

/// Write `line` plus CRLF. A CR or LF inside `line` (for a typed message,
/// e.g. in a string argument) would end it early and smuggle a second line
/// onto the wire, so that is refused.
fn write_line(line: String, dst: &mut BytesMut) -> Result<(), NetError> {
    if line.contains(['\r', '\n']) {
        return Err(NetError::Protocol(ippdme_core::IppError::InvalidArgument {
            name: "message".into(),
            reason: "must not contain CR or LF".into(),
        }));
    }
    dst.reserve(line.len() + 2);
    dst.put_slice(line.as_bytes());
    dst.put_slice(b"\r\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ippdme_core::Tag;

    #[test]
    fn decodes_one_line_at_a_time() {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::from("00001 # Ack()\r\n00002 ! Error(UnknownCommand)\r\n");
        let first = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(first.tag(), Tag(1));
        let second = codec.decode(&mut buf).unwrap().unwrap();
        assert_eq!(second.tag(), Tag(2));
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn decode_returns_none_for_partial_line() {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::from("00001 # A");
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn encode_rejects_line_breaks_inside_a_term() {
        use ippdme_core::Term;
        let msg = Message::Command {
            tag: Tag(1),
            term: Term::call(
                "LoadCoordSystem",
                vec![Term::Str("a\r\n00002 EndSession()".into())],
            ),
        };
        let mut buf = BytesMut::new();
        assert!(MessageCodec.encode(msg, &mut buf).is_err());
        assert!(buf.is_empty());
    }

    #[test]
    fn raw_lines_are_written_verbatim_but_stay_one_line() {
        let mut buf = BytesMut::new();
        let junk = Outgoing::RawLine("not a protocol line (".into());
        MessageCodec.encode(junk, &mut buf).unwrap();
        assert_eq!(&buf[..], b"not a protocol line (\r\n");

        let two_lines = Outgoing::RawLine("00001 Home()\r\n00002 EndSession()".into());
        assert!(MessageCodec.encode(two_lines, &mut buf).is_err());
    }

    #[test]
    fn encode_appends_crlf() {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::new();
        let msg = ippdme_core::response::ack(Tag(1));
        codec.encode(msg, &mut buf).unwrap();
        assert_eq!(&buf[..], b"00001 # Ack()\r\n");
    }
}
