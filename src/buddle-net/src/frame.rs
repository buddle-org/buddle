//! Representation of frames exchanged over the network.
//!
//! Frames either carry game-defined data or generic
//! control messages for session management.

use std::fmt;

use chrono::{DateTime, Utc};

use crate::control::{ClientKeepAlive, ControlMessage, ServerKeepAlive};

/// A data frame in the protocol.
#[derive(Clone, Debug, PartialEq)]
pub enum Frame {
    /// A control message.
    Control(ControlMessage),
}

impl Frame {
    // food + body_size
    pub(crate) const SMALL_HEADER_SIZE: usize = 2 + 2;
    // food + marker + body_size
    pub(crate) const LARGE_HEADER_SIZE: usize = 2 + 2 + 4;
    // is_control_message + opcode + reserved
    pub(crate) const BODY_SIZE: usize = 1 + 1 + 2;
    // service_id + order + payload_size
    pub(crate) const DML_HEADER_SIZE: usize = 1 + 1 + 2;

    pub(crate) fn binary_size(&self) -> usize {
        match self {
            Self::Control(ctrl) => ctrl.binary_size(),
        }
    }

    /// Constructs a new *Session Offer* frame given the
    /// required parameters.
    #[inline]
    pub fn session_offer(session_id: u16) -> Self {
        Self::Control(ControlMessage::SessionOffer {
            session_id,
            datetime: Utc::now(),
        })
    }

    /// Constructs a new *Session Accept* frame given the
    /// required parameters.
    #[inline]
    pub fn session_accept(session_id: u16) -> Self {
        Self::Control(ControlMessage::SessionAccept {
            session_id,
            datetime: Utc::now(),
        })
    }

    /// Constructs a new client-sided *Keep Alive* frame
    /// given the required parameters.
    #[inline]
    pub fn keep_alive(session_id: u16, session_start: DateTime<Utc>) -> Self {
        Self::Control(ControlMessage::ClientKeepAlive(ClientKeepAlive::new(
            session_id,
            session_start,
        )))
    }

    /// Constructs a new server-sided *Keep Alive Rsp* frame
    /// given the required parameters.
    #[inline]
    pub fn keep_alive_rsp(payload: ServerKeepAlive) -> Self {
        Self::Control(ControlMessage::ServerKeepAliveRsp(payload))
    }

    /// Whether this [`Frame`] is a control frame.
    #[inline]
    pub fn is_control(&self) -> bool {
        match self {
            Self::Control(..) => true,
        }
    }

    /// Gets the control opcode for this [`Frame`].
    ///
    /// This method returns [`None`] for data frames.
    #[inline]
    pub fn opcode(&self) -> Option<u8> {
        match self {
            Self::Control(ctrl) => Some(ctrl.opcode()),
        }
    }

    /// Whether this [`Frame`] is a data frame.
    #[inline]
    pub fn is_data(&self) -> bool {
        !self.is_control()
    }
}

impl From<ControlMessage> for Frame {
    fn from(value: ControlMessage) -> Self {
        Self::Control(value)
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Frame::Control(ctrl) => write!(f, "{ctrl}"),
        }
    }
}
