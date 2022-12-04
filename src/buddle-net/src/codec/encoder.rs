use std::io;

use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Utc};
use tokio_util::codec::Encoder;

use super::{is_large_frame, Codec, FOOD};
use crate::{control::*, frame::Frame};

const RESERVED: u16 = 0;

impl Encoder<Frame> for Codec {
    type Error = io::Error;

    fn encode(&mut self, frame: Frame, buf: &mut BytesMut) -> Result<(), Self::Error> {
        // Reserve enough space in `buf` so it fits the entire frame.
        // We optimistically reserve enough memory for a large header.
        let size = frame.binary_size();
        buf.reserve(Frame::LARGE_HEADER_SIZE + size);

        // Write the frame header.
        buf.put_u16_le(FOOD);
        if is_large_frame(size) {
            buf.put_u16_le(i16::MAX as u16 + 1);
            buf.put_u32_le(size as u32);
        } else {
            buf.put_u16_le(size as u16);
        }

        // Write the frame body and the message payload.
        write_frame_body(buf, frame.opcode());
        match frame {
            Frame::Control(ctrl) => {
                match ctrl {
                    ControlMessage::SessionOffer {
                        session_id,
                        datetime,
                    } => write_session_offer(buf, session_id, datetime),

                    ControlMessage::ClientKeepAlive(ka)
                    | ControlMessage::ClientKeepAliveRsp(ka) => write_client_keep_alive(buf, ka),

                    ControlMessage::ServerKeepAlive(ka)
                    | ControlMessage::ServerKeepAliveRsp(ka) => write_server_keep_alive(buf, ka),

                    ControlMessage::SessionAccept {
                        session_id,
                        datetime,
                    } => write_session_accept(buf, session_id, datetime),
                }
            }
        }

        Ok(())
    }
}

fn write_frame_body(buf: &mut BytesMut, opcode: Option<u8>) {
    buf.put_u8(opcode.is_some() as u8);
    buf.put_u8(opcode.unwrap_or(0));
    buf.put_u16_le(RESERVED);
}

fn write_session_offer(buf: &mut BytesMut, session_id: u16, datetime: DateTime<Utc>) {
    let (low_timestamp, high_timestamp) = split_timestamp(datetime.timestamp());

    buf.put_u16_le(session_id);
    buf.put_i32_le(high_timestamp);
    buf.put_i32_le(low_timestamp);
    buf.put_u32_le(datetime.timestamp_subsec_millis());
    // TODO: Crypto.
    buf.put_u8(0); // Trailing null byte.
}

fn write_client_keep_alive(buf: &mut BytesMut, data: ClientKeepAlive) {
    buf.put_u16_le(data.session_id);
    buf.put_u16_le(data.millis);
    buf.put_u16_le(data.minutes);
}

fn write_server_keep_alive(buf: &mut BytesMut, data: ServerKeepAlive) {
    buf.put_u16_le(INVALID_SESSION_ID);
    buf.put_u32_le(data.millis);
}

fn write_session_accept(buf: &mut BytesMut, session_id: u16, datetime: DateTime<Utc>) {
    let (low_timestamp, high_timestamp) = split_timestamp(datetime.timestamp());

    buf.put_u16_le(RESERVED);
    buf.put_i32_le(high_timestamp);
    buf.put_i32_le(low_timestamp);
    buf.put_u32_le(datetime.timestamp_subsec_millis());
    buf.put_u16_le(session_id);
    // TODO: Crypto.
    buf.put_u8(0); // Trailing null byte.
}

#[inline]
fn split_timestamp(timestamp: i64) -> (i32, i32) {
    // Strictly speaking, KI just discards the high half of
    // the timestamp which will lead to time calculations
    // breaking in 2038. This is a way for the community to
    // work around this limitation while still maintaining
    // compatibility with KI software as-is.
    (timestamp as i32, (timestamp >> 32) as i32)
}
