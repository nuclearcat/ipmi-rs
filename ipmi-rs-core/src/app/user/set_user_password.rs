use crate::connection::{CompletionErrorCode, IpmiCommand, Message, NetFn};

use super::UserId;

/// The stored size of a user password.
///
/// Reference: IPMI 2.0 Specification, Table 22-35 (byte 1, bit 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordSize {
    /// A 16-byte password/key (IPMI v1.5 backwards compatible).
    Bytes16,
    /// A 20-byte password/key (IPMI v2.0 / RMCP+).
    Bytes20,
}

impl PasswordSize {
    /// The length, in bytes, of a password of this size.
    pub fn byte_len(&self) -> usize {
        match self {
            PasswordSize::Bytes16 => 16,
            PasswordSize::Bytes20 => 20,
        }
    }
}

/// The operation performed by the Set User Password command.
///
/// Reference: IPMI 2.0 Specification, Table 22-35 (byte 2, bits [1:0]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    DisableUser,
    EnableUser,
    SetPassword,
    TestPassword,
}

impl Operation {
    fn value(&self) -> u8 {
        match self {
            Operation::DisableUser => 0b00,
            Operation::EnableUser => 0b01,
            Operation::SetPassword => 0b10,
            Operation::TestPassword => 0b11,
        }
    }
}

/// An error returned by the Set User Password command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetUserPasswordError {
    /// A `test password` operation failed: the password data does not match
    /// the stored value (completion code `0x80`).
    TestFailed,
    /// A `test password` operation failed: the wrong password size was used
    /// (completion code `0x81`).
    WrongPasswordSize,
}

/// The Set User Password command.
///
/// This command sets or changes user passwords, and enables or disables user
/// IDs. It can also be used to test a password against the stored value.
///
/// Reference: IPMI 2.0 Specification, Section 22.30, Table 22-35.
pub struct SetUserPassword {
    user_id: UserId,
    size: PasswordSize,
    operation: Operation,
    password: Vec<u8>,
}

impl SetUserPassword {
    /// Create a command that enables `user_id`.
    pub fn enable_user(user_id: UserId) -> Self {
        Self {
            user_id,
            size: PasswordSize::Bytes16,
            operation: Operation::EnableUser,
            password: Vec::new(),
        }
    }

    /// Create a command that disables `user_id`.
    pub fn disable_user(user_id: UserId) -> Self {
        Self {
            user_id,
            size: PasswordSize::Bytes16,
            operation: Operation::DisableUser,
            password: Vec::new(),
        }
    }

    /// Create a command that sets the password of `user_id`.
    ///
    /// `password` is the raw password octet string; it is null-padded to the
    /// length implied by `size`. Returns `None` if `password` is longer than
    /// that length.
    pub fn set_password(user_id: UserId, password: &[u8], size: PasswordSize) -> Option<Self> {
        Self::with_password(user_id, password, size, Operation::SetPassword)
    }

    /// Create a command that tests `password` against the stored password of
    /// `user_id`.
    ///
    /// `password` is null-padded to the length implied by `size`. Returns
    /// `None` if `password` is longer than that length.
    pub fn test_password(user_id: UserId, password: &[u8], size: PasswordSize) -> Option<Self> {
        Self::with_password(user_id, password, size, Operation::TestPassword)
    }

    fn with_password(
        user_id: UserId,
        password: &[u8],
        size: PasswordSize,
        operation: Operation,
    ) -> Option<Self> {
        if password.len() > size.byte_len() {
            return None;
        }

        let mut padded = vec![0u8; size.byte_len()];
        padded[..password.len()].copy_from_slice(password);

        Some(Self {
            user_id,
            size,
            operation,
            password: padded,
        })
    }
}

impl From<SetUserPassword> for Message {
    fn from(value: SetUserPassword) -> Self {
        let mut byte1 = value.user_id.value() & 0x3F;
        if value.size == PasswordSize::Bytes20 {
            byte1 |= 0x80;
        }

        let mut data = vec![byte1, value.operation.value()];
        data.extend_from_slice(&value.password);

        Message::new_request(NetFn::App, 0x47, data)
    }
}

impl IpmiCommand for SetUserPassword {
    type Output = ();
    type Error = SetUserPasswordError;

    fn handle_completion_code(
        completion_code: CompletionErrorCode,
        _data: &[u8],
    ) -> Option<Self::Error> {
        match completion_code {
            CompletionErrorCode::CommandSpecific(0x80) => Some(SetUserPasswordError::TestFailed),
            CompletionErrorCode::CommandSpecific(0x81) => {
                Some(SetUserPasswordError::WrongPasswordSize)
            }
            _ => None,
        }
    }

    fn parse_success_response(_data: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
