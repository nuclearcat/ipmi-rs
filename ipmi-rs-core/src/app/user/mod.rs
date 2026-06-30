//! Definitions for the IPMI user management commands.
//!
//! These commands operate on the management controller's user table: they
//! configure per-user channel access and privilege limits, user names, and
//! passwords.
//!
//! Reference: IPMI 2.0 Specification, Sections 22.26 - 22.30.

mod get_user_access;
pub use get_user_access::{GetUserAccess, UserAccess, UserIdEnableStatus};

mod set_user_access;
pub use set_user_access::SetUserAccess;

/// A user ID, used as the index into the management controller's user table.
///
/// Valid user IDs are in the range `1..=63`. User ID 1 is permanently
/// associated with the null user (`User 1`). The value `0` is reserved.
///
/// Reference: IPMI 2.0 Specification, Section 22.26.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(u8);

impl UserId {
    /// Create a new `UserId`.
    ///
    /// Returns `None` if `value` is `0` (reserved) or greater than `0x3F`, as
    /// the User ID field is only 6 bits wide.
    pub fn new(value: u8) -> Option<Self> {
        if value == 0 || value > 0x3F {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Get the numeric value of this `UserId`.
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl core::fmt::Display for UserId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "User {}", self.0)
    }
}

/// The privilege limit that applies to a user on a given channel.
///
/// This determines the maximum privilege level the user is allowed to switch
/// to on the channel. Unlike the session privilege levels, it includes the
/// `NoAccess` value (`0xF`).
///
/// Reference: IPMI 2.0 Specification, Table 22-31 and Table 22-32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserPrivilege {
    /// Callback level (`0x1`).
    Callback,
    /// User level (`0x2`).
    User,
    /// Operator level (`0x3`).
    Operator,
    /// Administrator level (`0x4`).
    Administrator,
    /// OEM Proprietary level (`0x5`).
    OemProprietary,
    /// No access (`0xF`).
    NoAccess,
    /// An unknown or reserved value.
    Unknown(u8),
}

impl From<u8> for UserPrivilege {
    fn from(value: u8) -> Self {
        match value & 0x0F {
            0x1 => Self::Callback,
            0x2 => Self::User,
            0x3 => Self::Operator,
            0x4 => Self::Administrator,
            0x5 => Self::OemProprietary,
            0xF => Self::NoAccess,
            v => Self::Unknown(v),
        }
    }
}

impl From<UserPrivilege> for u8 {
    fn from(value: UserPrivilege) -> Self {
        match value {
            UserPrivilege::Callback => 0x1,
            UserPrivilege::User => 0x2,
            UserPrivilege::Operator => 0x3,
            UserPrivilege::Administrator => 0x4,
            UserPrivilege::OemProprietary => 0x5,
            UserPrivilege::NoAccess => 0xF,
            UserPrivilege::Unknown(v) => v & 0x0F,
        }
    }
}

impl core::fmt::Display for UserPrivilege {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UserPrivilege::Callback => write!(f, "Callback"),
            UserPrivilege::User => write!(f, "User"),
            UserPrivilege::Operator => write!(f, "Operator"),
            UserPrivilege::Administrator => write!(f, "Administrator"),
            UserPrivilege::OemProprietary => write!(f, "OEM Proprietary"),
            UserPrivilege::NoAccess => write!(f, "No Access"),
            UserPrivilege::Unknown(v) => write!(f, "Unknown (0x{v:02X})"),
        }
    }
}
