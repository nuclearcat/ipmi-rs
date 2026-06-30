use crate::connection::{Channel, IpmiCommand, Message, NetFn, NotEnoughData};

use super::{UserId, UserPrivilege};

/// The Get User Access command.
///
/// This command retrieves the channel access information and enabled/disabled
/// state for a given user ID, as well as information about the number of
/// supported users on the channel.
///
/// Reference: IPMI 2.0 Specification, Section 22.27, Table 22-32.
pub struct GetUserAccess {
    channel: Channel,
    user_id: UserId,
}

impl GetUserAccess {
    /// Create a new Get User Access command for `user_id` on `channel`.
    pub fn new(channel: Channel, user_id: UserId) -> Self {
        Self { channel, user_id }
    }
}

impl From<GetUserAccess> for Message {
    fn from(value: GetUserAccess) -> Self {
        let channel = value.channel.value() & 0x0F;
        let user_id = value.user_id.value() & 0x3F;
        Message::new_request(NetFn::App, 0x44, vec![channel, user_id])
    }
}

impl IpmiCommand for GetUserAccess {
    type Output = UserAccess;
    type Error = NotEnoughData;

    fn parse_success_response(data: &[u8]) -> Result<Self::Output, Self::Error> {
        UserAccess::parse(data).ok_or(NotEnoughData)
    }
}

/// The enable status of a user ID, as reported by the Get User Access command.
///
/// Reference: IPMI 2.0 Specification, Table 22-32 (byte 3, bits [7:6]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserIdEnableStatus {
    /// Enable status unspecified (`00b`). Returned by pre-errata 3
    /// implementations for backwards compatibility.
    Unspecified,
    /// The user ID was enabled via the Set User Password command (`01b`).
    Enabled,
    /// The user ID was disabled via the Set User Password command (`10b`).
    Disabled,
    /// A reserved value (`11b`).
    Unknown,
}

impl From<u8> for UserIdEnableStatus {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => Self::Unspecified,
            0b01 => Self::Enabled,
            0b10 => Self::Disabled,
            _ => Self::Unknown,
        }
    }
}

/// User access information returned by the Get User Access command.
///
/// Reference: IPMI 2.0 Specification, Section 22.27, Table 22-32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserAccess {
    /// Maximum number of user IDs supported on this channel (1-based, includes
    /// User 1).
    pub max_user_ids: u8,
    /// Count of currently enabled user IDs on this channel.
    pub enabled_user_count: u8,
    /// The enable status of the requested user ID.
    pub enable_status: UserIdEnableStatus,
    /// Count of user IDs with fixed names, including User 1.
    pub fixed_name_count: u8,
    /// `true` if user access is available only during a callback connection.
    pub callback_only: bool,
    /// `true` if the user is enabled for link authentication.
    pub link_auth_enabled: bool,
    /// `true` if the user is enabled for IPMI messaging.
    pub ipmi_messaging_enabled: bool,
    /// The privilege limit for the user on this channel.
    pub privilege_limit: UserPrivilege,
}

impl UserAccess {
    /// Parse a `UserAccess` from IPMI response data.
    ///
    /// Reference: IPMI 2.0 Specification, Table 22-32.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let max_user_ids = data[0] & 0x3F;
        let enable_status = UserIdEnableStatus::from((data[1] >> 6) & 0b11);
        let enabled_user_count = data[1] & 0x3F;
        let fixed_name_count = data[2] & 0x3F;

        let callback_only = (data[3] & 0x40) == 0x40;
        let link_auth_enabled = (data[3] & 0x20) == 0x20;
        let ipmi_messaging_enabled = (data[3] & 0x10) == 0x10;
        let privilege_limit = UserPrivilege::from(data[3] & 0x0F);

        Some(Self {
            max_user_ids,
            enabled_user_count,
            enable_status,
            fixed_name_count,
            callback_only,
            link_auth_enabled,
            ipmi_messaging_enabled,
            privilege_limit,
        })
    }
}
