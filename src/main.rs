#![feature(setgroups)]

use std::fs::File;
use std::io::Read;

use std::{os::unix::process::CommandExt, process::Command};

use etc_passwd::Passwd;

mod config;
mod handler;
mod password;

const SERVICE_NAME: &str = "odo";
const SAFE_PATH: &str = "/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin:/usr/local/sbin";

fn get_config() -> std::io::Result<config::Config> {
    let mut file = File::open("odo.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(toml::from_str(&contents)?)
}

fn get_command() -> Command {
    let mut args = std::env::args();
    args.next();

    let mut command = Command::new(args.next().expect("No command provided"));
    command.args(args);

    command
}

fn init_pam(
    username: &str,
    mask: Option<char>,
) -> pam::Authenticator<'static, handler::ConvHandler> {
    let handler = handler::ConvHandler::new(username, mask);

    pam::Authenticator::with_handler(SERVICE_NAME, handler).unwrap()
}

fn check_identity(username: &str, groups: &[users::Group], identity: &str) -> bool {
    let mut identity_iter = identity.chars();

    if identity_iter.next().unwrap() == ':' {
        let identity_group = identity_iter.as_str();
        let mut matched = false;

        for group in groups {
            if group.name() == identity_group {
                matched = true;
                break;
            }
        }

        matched
    } else {
        username == identity
    }
}

fn main() {
    let config = get_config().unwrap();

    let username = users::get_current_username()
        .unwrap()
        .into_string()
        .unwrap();

    let groups: Vec<users::Group> = users::get_user_groups(&username, users::get_current_gid())
        .unwrap()
        .into_iter()
        .filter(|g| g.gid() != users::get_effective_gid()) // Current effective gid should not be considered
        .collect();

    let mut command = get_command();

    for rule in config.rule {
        if check_identity(&username, &groups, &rule.identity)
            && rule.commands.map_or(true, |c| {
                c.contains(&command.get_program().to_str().unwrap().to_string())
            })
        {
            // Ask for auth if required
            if rule.auth.unwrap_or(true) {
                let mut auth = init_pam(&username, config.mask);

                if auth.authenticate().is_err() || auth.open_session().is_err() {
                    println!("Authentication failed!");
                    std::process::exit(1);
                }
            }

            let user = if let Some(target) = rule.r#as {
                users::get_user_by_name(&target).expect("Invalid user")
            } else {
                users::get_user_by_uid(0).expect("Failed to get root user")
            };

            // Set new UID and groups of process
            command.uid(user.uid());
            command.gid(user.primary_group_id());

            // Potentially should be reworked
            command.groups(
                &user
                    .groups()
                    .expect("Failed to read users groups")
                    .into_iter()
                    .map(|g| g.gid())
                    .filter(|g| *g != users::get_effective_gid()) // Remove current effective gid
                    .collect::<Vec<u32>>(),
            );

            // Set environment
            if !rule.keepenv.unwrap_or(false) {
                command.env_clear();
            }

            let passwd = Passwd::from_uid(user.uid()).unwrap().unwrap();

            command.env("PATH", SAFE_PATH);
            command.env("LOGNAME", user.name());
            command.env("USER", user.name());
            command.env("SHELL", passwd.shell.to_str().unwrap());

            if let Ok(term) = std::env::var("TERM") {
                command.env("TERM", term);
            }

            if let Ok(display) = std::env::var("DISPLAY") {
                command.env("DISPLAY", display);
            }

            command.env("DOAS_USER", users::get_current_username().unwrap());

            // Execute command, exec() call will not return unless there is an error
            println!("Failed to execute command: {:?}", command.exec());
            std::process::exit(1);
        }
    }

    println!("No rules matched!");
}
