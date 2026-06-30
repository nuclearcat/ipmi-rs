use crate::connection::{IpmiCommand, Message, NetFn, NotEnoughData};

use super::UserId;

/// The maximum length, in bytes, of a user name.
pub const MAX_USER_NAME_LEN: usize = 16;

/// The Set User Name command.
///
/// This command assigns a user name to a given user ID. The name is stored as
/// a 16-byte ASCII string, null-terminated and null-padded. There is no
/// configurable name for User ID 1, which is permanently the null user.
///
/// Reference: IPMI 2.0 Specification, Section 22.28, Table 22-33.
pub struct SetUserName {
    user_id: UserId,
    name: [u8; MAX_USER_NAME_LEN],
}

impl SetUserName {
    /// Create a new Set User Name command setting the name of `user_id` to
    /// `name`.
    ///
    /// Returns `None` if `name` is longer than [`MAX_USER_NAME_LEN`] bytes.
    pub fn new(user_id: UserId, name: &str) -> Option<Self> {
        let bytes = name.as_bytes();
        if bytes.len() > MAX_USER_NAME_LEN {
            return None;
        }

        let mut name = [0u8; MAX_USER_NAME_LEN];
        name[..bytes.len()].copy_from_slice(bytes);

        Some(Self { user_id, name })
    }
}

impl From<SetUserName> for Message {
    fn from(value: SetUserName) -> Self {
        let mut data = Vec::with_capacity(1 + MAX_USER_NAME_LEN);
        data.push(value.user_id.value() & 0x3F);
        data.extend_from_slice(&value.name);
        Message::new_request(NetFn::App, 0x45, data)
    }
}

impl IpmiCommand for SetUserName {
    type Output = ();
    type Error = NotEnoughData;

    fn parse_success_response(_data: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
