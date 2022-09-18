use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Action {
    /// Diff two api response base on given profiles
    Run(RunArgs),
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct RunArgs {
    /// profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// param overrides
    #[clap(short,long,value_parser=parse_key_val, number_of_values=1)]
    extra_params: Vec<KeyVal>,
}

#[derive(Debug, Clone)]
pub(crate) struct KeyVal {
    key: String,
    val: String,
}

pub(crate) enum KeyValType {
    Query,
    Header,
    Body,
}

fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');

    fn receiver<'a>(x: Option<&'a str>) -> Result<&'a str> {
        x.ok_or_else(|| anyhow!("error"))
    }

    let key = receiver(parts.next())?;
    let value = receiver(parts.next())?;

    let key_type = match key.chars().next() {
        Some('%') => KeyValType::Header,
        Some('@') => KeyValType::Body,
        _ => KeyValType::Query,
    };

    let key = match key_type {
        KeyValType::Header => key[1..].to_string(),
        KeyValType::Body => key[1..].to_string(),
        KeyValType::Query => key[1..].to_string(),
    };

    todo!()
}
