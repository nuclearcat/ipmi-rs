//! User management tool example
//!
//! This example demonstrates the IPMI user management commands: listing users
//! and their access, setting user access and privilege limits, setting user
//! names, and setting/enabling/disabling/testing user passwords.
//!
//! Usage:
//!   # List all users on the current channel
//!   cargo run --example ipmi-users -- list
//!
//!   # Set user 2 to Operator privilege, enabled for IPMI messaging
//!   cargo run --example ipmi-users -- set-access --user 2 --privilege operator --ipmi-messaging
//!
//!   # Set the name and password of user 2, then enable it
//!   cargo run --example ipmi-users -- set-name --user 2 --name admin
//!   cargo run --example ipmi-users -- set-password --user 2 --password hunter2
//!   cargo run --example ipmi-users -- enable --user 2

mod common;

use clap::{Parser, Subcommand, ValueEnum};
use ipmi_rs::{
    app::{
        GetUserAccess, GetUserName, PasswordSize, SetUserAccess, SetUserName, SetUserPassword,
        UserAccess, UserId, UserPrivilege,
    },
    connection::Channel,
};

#[derive(Parser)]
struct Command {
    #[clap(flatten)]
    common: common::CommonOpts,

    /// The channel to operate on (0x0 - 0xF). Defaults to the current channel.
    #[clap(long, default_value = "0xE", value_parser = parse_channel, global = true)]
    channel: Channel,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// List all users on the channel and their access.
    List,
    /// Configure a user's privilege limit and channel access.
    SetAccess(SetAccessArgs),
    /// Set a user's name.
    SetName(SetNameArgs),
    /// Set a user's password.
    SetPassword(SetPasswordArgs),
    /// Enable a user.
    Enable(UserArg),
    /// Disable a user.
    Disable(UserArg),
    /// Test a user's password against the stored value.
    TestPassword(SetPasswordArgs),
}

#[derive(Parser)]
struct UserArg {
    /// The user ID (1 - 63).
    #[clap(long, value_parser = parse_user_id)]
    user: UserId,
}

#[derive(Parser)]
struct SetAccessArgs {
    /// The user ID (1 - 63).
    #[clap(long, value_parser = parse_user_id)]
    user: UserId,
    /// The maximum privilege level the user may switch to.
    #[clap(long, value_enum)]
    privilege: Privilege,
    /// Enable the user for IPMI messaging.
    #[clap(long)]
    ipmi_messaging: bool,
    /// Enable the user for link authentication.
    #[clap(long)]
    link_auth: bool,
    /// Restrict the user to Callback level for non-callback connections.
    #[clap(long)]
    callback_restricted: bool,
}

#[derive(Parser)]
struct SetNameArgs {
    /// The user ID (1 - 63).
    #[clap(long, value_parser = parse_user_id)]
    user: UserId,
    /// The name to set (max 16 bytes).
    #[clap(long)]
    name: String,
}

#[derive(Parser)]
struct SetPasswordArgs {
    /// The user ID (1 - 63).
    #[clap(long, value_parser = parse_user_id)]
    user: UserId,
    /// The password (max 16 bytes, or 20 with --bytes20).
    #[clap(long)]
    password: String,
    /// Use a 20-byte password (IPMI v2.0 / RMCP+) instead of 16 bytes.
    #[clap(long)]
    bytes20: bool,
}

/// The privilege levels selectable on the command line.
#[derive(Clone, Copy, ValueEnum)]
enum Privilege {
    Callback,
    User,
    Operator,
    Administrator,
    Oem,
    NoAccess,
}

impl From<Privilege> for UserPrivilege {
    fn from(value: Privilege) -> Self {
        match value {
            Privilege::Callback => UserPrivilege::Callback,
            Privilege::User => UserPrivilege::User,
            Privilege::Operator => UserPrivilege::Operator,
            Privilege::Administrator => UserPrivilege::Administrator,
            Privilege::Oem => UserPrivilege::OemProprietary,
            Privilege::NoAccess => UserPrivilege::NoAccess,
        }
    }
}

fn parse_channel(value: &str) -> Result<Channel, String> {
    let raw = parse_u8(value)?;
    Channel::new(raw).ok_or_else(|| format!("invalid channel number 0x{raw:02X}"))
}

fn parse_user_id(value: &str) -> Result<UserId, String> {
    let raw = parse_u8(value)?;
    UserId::new(raw).ok_or_else(|| format!("invalid user ID {raw} (must be 1 - 63)"))
}

fn parse_u8(value: &str) -> Result<u8, String> {
    let result = if let Some(hex) = value.strip_prefix("0x") {
        u8::from_str_radix(hex, 16)
    } else {
        value.parse()
    };
    result.map_err(|_| format!("invalid number: {value}"))
}

fn password_size(bytes20: bool) -> PasswordSize {
    if bytes20 {
        PasswordSize::Bytes20
    } else {
        PasswordSize::Bytes16
    }
}

fn print_user(id: UserId, name: &str, access: &UserAccess) {
    let name = if name.is_empty() { "<unnamed>" } else { name };
    println!("User {} ({})", id.value(), name);
    println!("  Privilege limit: {}", access.privilege_limit);
    println!("  IPMI messaging:  {}", access.ipmi_messaging_enabled);
    println!("  Link auth:       {}", access.link_auth_enabled);
    println!("  Callback only:   {}", access.callback_only);
}

fn list(ipmi: &mut common::IpmiConnectionEnum, channel: Channel) -> std::io::Result<()> {
    // User 1 always exists; use it to discover how many user IDs are supported.
    let first = UserId::new(1).expect("user ID 1 is valid");
    let info = ipmi
        .send_recv(GetUserAccess::new(channel, first))
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Get User Access failed: {e:?}"),
            )
        })?;

    println!(
        "{} supports {} user IDs, {} currently enabled\n",
        channel, info.max_user_ids, info.enabled_user_count
    );

    for raw in 1..=info.max_user_ids {
        let user = match UserId::new(raw) {
            Some(user) => user,
            None => continue,
        };

        let access = match ipmi.send_recv(GetUserAccess::new(channel, user)) {
            Ok(access) => access,
            Err(e) => {
                log::warn!("Get User Access failed for user {raw}: {e:?}");
                continue;
            }
        };

        let name = ipmi.send_recv(GetUserName::new(user)).unwrap_or_default();

        print_user(user, &name, &access);
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::formatted_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or("info".to_string()))
        .init();

    let command = Command::parse();
    let channel = command.channel;
    let mut ipmi = command.common.get_connection()?;

    match command.action {
        Action::List => list(&mut ipmi, channel)?,
        Action::SetAccess(args) => {
            let request = SetUserAccess::new(channel, args.user, args.privilege.into())
                .ipmi_messaging_enabled(args.ipmi_messaging)
                .link_auth_enabled(args.link_auth)
                .callback_restricted(args.callback_restricted);
            ipmi.send_recv(request).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Set User Access failed: {e:?}"),
                )
            })?;
            println!("User {} access updated", args.user.value());
        }
        Action::SetName(args) => {
            let request = SetUserName::new(args.user, &args.name).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "user name too long (max 16 bytes)",
                )
            })?;
            ipmi.send_recv(request).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Set User Name failed: {e:?}"),
                )
            })?;
            println!("User {} name set to '{}'", args.user.value(), args.name);
        }
        Action::SetPassword(args) => {
            let size = password_size(args.bytes20);
            let request = SetUserPassword::set_password(args.user, args.password.as_bytes(), size)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "password too long for the selected size",
                    )
                })?;
            ipmi.send_recv(request).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Set User Password failed: {e:?}"),
                )
            })?;
            println!("User {} password set", args.user.value());
        }
        Action::Enable(args) => {
            ipmi.send_recv(SetUserPassword::enable_user(args.user))
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Enable user failed: {e:?}"),
                    )
                })?;
            println!("User {} enabled", args.user.value());
        }
        Action::Disable(args) => {
            ipmi.send_recv(SetUserPassword::disable_user(args.user))
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Disable user failed: {e:?}"),
                    )
                })?;
            println!("User {} disabled", args.user.value());
        }
        Action::TestPassword(args) => {
            let size = password_size(args.bytes20);
            let request = SetUserPassword::test_password(args.user, args.password.as_bytes(), size)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "password too long for the selected size",
                    )
                })?;
            match ipmi.send_recv(request) {
                Ok(()) => println!("Password for user {} matches", args.user.value()),
                Err(e) => println!("Password test failed: {e:?}"),
            }
        }
    }

    Ok(())
}
