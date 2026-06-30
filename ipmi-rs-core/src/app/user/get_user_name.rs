use crate::connection::{IpmiCommand, Message, NetFn, NotEnoughData};

use super::{set_user_name::MAX_USER_NAME_LEN, UserId};

/// The Get User Name command.
///
/// This command retrieves the user name that was set for a given user ID using
/// the Set User Name command.
///
/// Reference: IPMI 2.0 Specification, Section 22.29, Table 22-34.
pub struct GetUserName {
    user_id: UserId,
}

impl GetUserName {
    /// Create a new Get User Name command for `user_id`.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

impl From<GetUserName> for Message {
    fn from(value: GetUserName) -> Self {
        Message::new_request(NetFn::App, 0x46, vec![value.user_id.value() & 0x3F])
    }
}

impl IpmiCommand for GetUserName {
    type Output = String;
    type Error = NotEnoughData;

    fn parse_success_response(data: &[u8]) -> Result<Self::Output, Self::Error> {
        if data.len() < MAX_USER_NAME_LEN {
            return Err(NotEnoughData);
        }

        // The name is null-terminated; only the bytes up to the first null are
        // part of the name.
        let end = data[..MAX_USER_NAME_LEN]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_USER_NAME_LEN);

        Ok(String::from_utf8_lossy(&data[..end]).into_owned())
    }
}
