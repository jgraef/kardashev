mod constants;
mod galaxy;
mod time_scale;

use std::{
    collections::BTreeSet,
    fs::File,
    io::BufReader,
};

use clap::Parser;
pub use color_eyre::eyre::Error;

use crate::time_scale::time_scale;

#[derive(Debug, Parser)]
enum Args {
    TimeScale { time_per_year: Vec<String> },
    KeyCodes,
}

fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    match args {
        Args::TimeScale { time_per_year } => time_scale(time_per_year)?,
        Args::KeyCodes => key_codes()?,
    }

    Ok(())
}

fn key_codes() -> Result<(), Error> {
    let s = include_str!("../key_codes.txt");
    let mut codes = BTreeSet::new();
    for line in s.lines() {
        let line = line.trim();
        if !line.is_empty() {
            let parts = line.split('\t').collect::<Vec<_>>();
            if let Some(name) = parts.get(1).and_then(|name| str_between(name, "\"", "\"")) {
                if name != "Unidentified" {
                    codes.insert(name);
                }
            }
        }
    }

    println!("key_codes!{{");
    for code in codes {
        println!("    {code},");
    }
    println!("}}");

    Ok(())
}

fn str_between<'a>(s: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start = s.find(start)? + start.len();
    let end = s[start..].find(end)?;
    Some(&s[start..=end])
}
