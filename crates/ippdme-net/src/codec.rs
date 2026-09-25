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

impl Encoder<Message> for MessageCodec {
    type Error = NetError;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), NetError> {
        let line = item.to_string();
        dst.reserve(line.len() + 2);
        dst.put_slice(line.as_bytes());
        dst.put_slice(b"\r\n");
        Ok(())
    }
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
    fn encode_appends_crlf() {
        let mut codec = MessageCodec;
        let mut buf = BytesMut::new();
        let msg = ippdme_core::response::ack(Tag(1));
        codec.encode(msg, &mut buf).unwrap();
        assert_eq!(&buf[..], b"00001 # Ack()\r\n");
    }
}
