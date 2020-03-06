use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::result;

use alphred::{Item, Workflow};
use anyhow::{anyhow, Result};

// Ported from https://github.com/aiyodk/Alfred-Extensions

fn main() -> Result<()> {
    // let guest_enabled = run("defaults", &["read", "/Library/Preferences/com.apple.loginwindow", "GuestEnabled"])?.parse::<usize>()? == 1;
    // let me = run("whoami", &[])?;
    let output = run(
        "dscl",
        &[".", "-list", "/Users", "_writers_UserCertificate"],
    )?;
    let users: Vec<_> = output
        .lines()
        .map(|l| l.split_whitespace().next())
        .collect::<Option<_>>()
        .ok_or_else(|| anyhow!("user"))?;

    let mut items = vec![];
    for user in users {
        let icon_path = icon(user).unwrap_or_else(|_| PathBuf::from("icon.png"));
        let item = Item::new(user).arg(user).icon(icon_path.as_path());
        items.push(item);
    }
    println!("{}", Workflow::new(&items));

    Ok(())
}

fn run(program: &str, args: &[&str]) -> Result<String> {
    let stdout = Command::new(program).args(args).output()?.stdout;
    let output = String::from_utf8(stdout)?.trim().to_string();
    Ok(output)
}

fn icon(username: &str) -> Result<PathBuf> {
    let mut path = Workflow::cache()?;
    path.push(username);
    path.set_extension("jpg");

    if path.exists() {
        return Ok(path);
    }

    let binary = read_icon(username)?;
    let mut buffer = File::create(&path)?;
    buffer.write_all(&binary)?;

    Ok(path)
}

fn read_icon(username: &str) -> Result<Vec<u8>> {
    let photo = run(
        "dscl",
        &[".", "-read", &format!("/Users/{}/", username), "JPEGPhoto"],
    )?;
    // fallback: dscl . -read "/Users/$USERNAME/" Picture

    let hex: Vec<_> = photo
        .lines()
        .last()
        .ok_or_else(|| anyhow!("couldn't read user {} photo", username))?
        .replace(" ", "")
        .chars()
        .collect();

    let binary: Vec<_> = hex
        .chunks(2)
        .map(|w| u8::from_str_radix(&format!("{}{}", w[0], w[1]), 16))
        .collect::<result::Result<_, _>>()?;

    Ok(binary)
}
