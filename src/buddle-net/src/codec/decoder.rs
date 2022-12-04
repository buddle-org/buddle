use std::io;

use bytes::{Buf, BytesMut};
use chrono::{TimeZone, Utc};
use tokio_util::codec::Decoder;

use super::{is_large_frame, Codec, FOOD};
use crate::{control::*, frame::Frame};

impl Decoder for Codec {
    type Item = Frame;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // If less than the first header bytes are available, read more.
        // We optimistically assume the frame is gonna be a large one.
        if buf.len() < Frame::LARGE_HEADER_SIZE {
            return Ok(None);
        }

        // Read and validate the header magic of every frame.
        if buf.get_u16_le() != FOOD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frame does not start with food magic",
            ));
        }

        // Determine the size of the current frame.
        let body_size = {
            let size = buf.get_u16_le() as usize;
            if is_large_frame(size) {
                buf.get_u32_le() as usize
            } else {
                size
            }
        };

        // Reserve enough memory to complete decoding the current frame
        // + the next frame's header and wait for the bytes to arrive.
        buf.reserve(body_size + Frame::LARGE_HEADER_SIZE);
        if body_size > buf.len() {
            return Ok(None);
        }

        // Read the frame body.
        let is_control_frame = buf.get_u8() != 0;
        let opcode = is_control_frame.then_some(buf.get_u8());
        buf.get_u16(); // Reserved.

        // Read the payload depending on the body type.
        if let Some(opcode) = opcode {
            let frame = Frame::Control(read_control_message(buf, opcode)?);
            Ok(Some(frame))
        } else {
            todo!()
        }
    }
}

fn read_control_message(buf: &mut BytesMut, opcode: u8) -> io::Result<ControlMessage> {
    match opcode {
        OP_SESSION_OFFER => Ok(read_session_offer(buf)),
        OP_KEEP_ALIVE => Ok(read_keep_alive(buf)),
        OP_KEEP_ALIVE_RSP => Ok(read_keep_alive_rsp(buf)),
        OP_SESSION_ACCEPT => Ok(read_session_accept(buf)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid control opcode: {opcode}"),
        )),
    }
}

fn read_session_offer(buf: &mut BytesMut) -> ControlMessage {
    let session_id = buf.get_u16_le();
    let datetime = Utc
        .timestamp_opt(
            combine_timestamp(buf.get_i32_le(), buf.get_i32_le()),
            buf.get_u32_le() * 1_000_000, // milliseconds to microseconds
        )
        .unwrap();
    buf.get_u8(); // Discard trailing null byte.

    ControlMessage::SessionOffer {
        session_id,
        datetime,
    }
}

fn read_keep_alive(buf: &mut BytesMut) -> ControlMessage {
    let session_id = buf.get_u16_le();

    if session_id == INVALID_SESSION_ID {
        ControlMessage::ServerKeepAlive(ServerKeepAlive {
            millis: buf.get_u32_le(),
        })
    } else {
        ControlMessage::ClientKeepAlive(ClientKeepAlive {
            session_id,
            millis: buf.get_u16_le(),
            minutes: buf.get_u16_le(),
        })
    }
}

fn read_keep_alive_rsp(buf: &mut BytesMut) -> ControlMessage {
    let session_id = buf.get_u16_le();

    if session_id == INVALID_SESSION_ID {
        ControlMessage::ServerKeepAliveRsp(ServerKeepAlive {
            millis: buf.get_u32_le(),
        })
    } else {
        ControlMessage::ClientKeepAliveRsp(ClientKeepAlive {
            session_id,
            millis: buf.get_u16_le(),
            minutes: buf.get_u16_le(),
        })
    }
}

fn read_session_accept(buf: &mut BytesMut) -> ControlMessage {
    buf.get_u16_le(); // Reserved.
    let datetime = Utc
        .timestamp_opt(
            combine_timestamp(buf.get_i32_le(), buf.get_i32_le()),
            buf.get_u32_le() * 1_000_000, // milliseconds to microseconds
        )
        .unwrap();
    let session_id = buf.get_u16_le();
    buf.get_u8(); // Discard trailing null byte.

    ControlMessage::SessionAccept {
        session_id,
        datetime,
    }
}

#[inline]
fn combine_timestamp(high: i32, low: i32) -> i64 {
    (high as i64) << 32 | (low as i64) & 0xFFFF_FFFF
}
