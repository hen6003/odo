use std::fs::File;
use std::io::Read;

use std::{os::unix::process::CommandExt, process::Command};

mod config;
mod handler;

const SERVICE_NAME: &str = "odo";

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

fn init_pam(username: &str) -> pam::Authenticator<'static, handler::ConvHandler> {
    let handler = handler::ConvHandler::new(username);

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
            let mut auth = init_pam(&username);

            if rule.password.unwrap_or(true)
                && (auth.authenticate().is_err() || auth.open_session().is_err())
            {
                println!("Authentication failed!");
            }

            println!("Failed to execute command: {:?}", command.exec());
            std::process::exit(1);
        }
    }

    println!("No rules matched!");
}