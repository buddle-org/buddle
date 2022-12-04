//! Implementation of control messages used for connection
//! management.

use std::{fmt, time::Duration};

use chrono::{DateTime, Utc};

/// An invalid Session ID value that may never be
/// proposed to clients.
///
/// Any other value in the [`u16`] range is valid.
pub const INVALID_SESSION_ID: u16 = 0;

/// The opcode for Session Offer control messages.
pub const OP_SESSION_OFFER: u8 = 0x0;
/// The opcode for Keep Alive control messages.
///
/// This is used for both directions.
pub const OP_KEEP_ALIVE: u8 = 0x3;
/// The opcode for Keep Alive Rsp control messages.
///
/// This is used for both directions.
pub const OP_KEEP_ALIVE_RSP: u8 = 0x4;
/// The opcode for Session Accept control messages.
pub const OP_SESSION_ACCEPT: u8 = 0x5;

/// The periodic interval between which a server should
/// send Keep Alive messages to clients.
pub const SERVER_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

/// The periodic interval between which clients should
/// send Keep Alive messages to a server.
pub const CLIENT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

/// The data payload of client-sided Keep Alive messages.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientKeepAlive {
    /// The Session ID assigned to the client.
    pub session_id: u16,
    /// The milliseconds into the second the message
    /// was constructed at.
    pub millis: u16,
    /// The minutes that have elapsed since the session
    /// was started.
    pub minutes: u16,
}

impl ClientKeepAlive {
    /// Creates a new client-sided Keep Alive payload
    /// given the raw session details.
    pub fn new(session_id: u16, session_start: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            millis: now.timestamp_subsec_millis() as u16,
            minutes: (now - session_start).num_minutes() as u16,
        }
    }
}

/// The data payload of server-sided Keep Alive messages.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerKeepAlive {
    /// The number of milliseconds since the server
    /// was started.
    pub millis: u32,
}

/// Representation of various control messages sent over
/// the network for connection management.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ControlMessage {
    /// Representation of a Session Offer message.
    SessionOffer {
        /// The suggested Session ID to agree on.
        session_id: u16,
        /// The point in time at which the message was
        /// constructed.
        datetime: DateTime<Utc>,
    },

    /// Representation of a server-sided Keep Alive
    /// message.
    ServerKeepAlive(ServerKeepAlive),

    /// Representation of a client-sided Keep Alive
    /// message.
    ClientKeepAlive(ClientKeepAlive),

    /// Representation of a server-sided Keep Alive Rsp
    /// message.
    ServerKeepAliveRsp(ServerKeepAlive),

    /// Representation of a client-sided Keep Alive Rsp
    /// message.
    ClientKeepAliveRsp(ClientKeepAlive),

    /// Representation of a Session Accept message.
    SessionAccept {
        /// Echo of the proposed Session ID for confirmation.
        session_id: u16,
        /// The point in time at which the message was
        /// constructed.
        datetime: DateTime<Utc>,
    },
}

impl ControlMessage {
    // session_id + timestamp + subsec_millis + null
    pub(crate) const SESSION_OFFER_SIZE: usize = 2 + 8 + 4 + 1;
    // session_id + time
    pub(crate) const KEEP_ALIVE_SIZE: usize = 2 + 4;
    // reserved + timestamp + subsec_millis + session_id + null
    pub(crate) const SESSION_ACCEPT_SIZE: usize = 2 + 8 + 4 + 2 + 1;

    /// Gets the control opcode value that corresponds
    /// to `self`.
    pub fn opcode(&self) -> u8 {
        use ControlMessage::*;
        match self {
            SessionOffer { .. } => OP_SESSION_OFFER,
            ClientKeepAlive(..) | ServerKeepAlive(..) => OP_KEEP_ALIVE,
            ClientKeepAliveRsp(..) | ServerKeepAliveRsp(..) => OP_KEEP_ALIVE_RSP,
            SessionAccept { .. } => OP_SESSION_ACCEPT,
        }
    }

    pub(crate) fn binary_size(&self) -> usize {
        use ControlMessage::*;
        match self {
            SessionOffer { .. } => Self::SESSION_OFFER_SIZE,

            ClientKeepAlive(..)
            | ClientKeepAliveRsp(..)
            | ServerKeepAlive(..)
            | ServerKeepAliveRsp(..) => Self::KEEP_ALIVE_SIZE,

            SessionAccept { .. } => Self::SESSION_ACCEPT_SIZE,
        }
    }
}

impl fmt::Display for ControlMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ControlMessage::*;
        match self {
            SessionOffer { session_id, .. } => write!(f, "Session Offer ({session_id})"),
            ClientKeepAlive(ka) => write!(f, "Client Keep Alive ({})", ka.session_id),
            ServerKeepAlive(ka) => write!(f, "Server Keep Alive ({})", ka.millis),
            ClientKeepAliveRsp(ka) => write!(f, "Client Keep Alive Rsp ({})", ka.session_id),
            ServerKeepAliveRsp(ka) => write!(f, "Server Keep Alive Rsp ({})", ka.millis),
            SessionAccept { session_id, .. } => write!(f, "Session Accept ({session_id})"),
        }
    }
}
