use crate::connection::{Channel, IpmiCommand, Message, NetFn, NotEnoughData};

use super::{UserId, UserPrivilege};

/// The Set User Access command.
///
/// This command configures the privilege limit and channel accessibility
/// associated with a given user ID. The `change_bits` flag in the request is
/// always set, so the link authentication, IPMI messaging and callback
/// settings provided here are always applied.
///
/// Reference: IPMI 2.0 Specification, Section 22.26, Table 22-31.
pub struct SetUserAccess {
    channel: Channel,
    user_id: UserId,
    callback_restricted: bool,
    link_auth_enabled: bool,
    ipmi_messaging_enabled: bool,
    privilege_limit: UserPrivilege,
    session_limit: Option<u8>,
}

impl SetUserAccess {
    /// Create a new Set User Access command for `user_id` on `channel`,
    /// restricting the user to at most `privilege_limit`.
    pub fn new(channel: Channel, user_id: UserId, privilege_limit: UserPrivilege) -> Self {
        Self {
            channel,
            user_id,
            callback_restricted: false,
            link_auth_enabled: false,
            ipmi_messaging_enabled: false,
            privilege_limit,
            session_limit: None,
        }
    }

    /// Restrict the user to Callback level for non-callback connections.
    pub fn callback_restricted(mut self, value: bool) -> Self {
        self.callback_restricted = value;
        self
    }

    /// Enable or disable the user for link authentication on this channel.
    pub fn link_auth_enabled(mut self, value: bool) -> Self {
        self.link_auth_enabled = value;
        self
    }

    /// Enable or disable the user for IPMI messaging on this channel.
    pub fn ipmi_messaging_enabled(mut self, value: bool) -> Self {
        self.ipmi_messaging_enabled = value;
        self
    }

    /// Set the optional user session limit (the maximum number of simultaneous
    /// sessions for this user). Only the low 4 bits are significant.
    pub fn session_limit(mut self, limit: u8) -> Self {
        self.session_limit = Some(limit);
        self
    }
}

impl From<SetUserAccess> for Message {
    fn from(value: SetUserAccess) -> Self {
        // Bit 7 enables changing the remaining bits in this byte.
        let mut byte1 = 0x80 | (value.channel.value() & 0x0F);
        if value.callback_restricted {
            byte1 |= 0x40;
        }
        if value.link_auth_enabled {
            byte1 |= 0x20;
        }
        if value.ipmi_messaging_enabled {
            byte1 |= 0x10;
        }

        let user_id = value.user_id.value() & 0x3F;
        let privilege = u8::from(value.privilege_limit) & 0x0F;

        let mut data = vec![byte1, user_id, privilege];
        if let Some(limit) = value.session_limit {
            data.push(limit & 0x0F);
        }

        Message::new_request(NetFn::App, 0x43, data)
    }
}

impl IpmiCommand for SetUserAccess {
    type Output = ();
    type Error = NotEnoughData;

    fn parse_success_response(_data: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
