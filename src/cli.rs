use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use crate::ExtraArgs;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// Diff two api response base on given profiles
    Run(RunArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// param overrides
    #[clap(short,long,value_parser=parse_key_val, number_of_values=1)]
    pub extra_params: Vec<KeyVal>,

    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KeyVal {
    key_type: KeyValType,
    key: String,
    val: String,
}

#[derive(Debug, Clone)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValList {}

fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');

    fn retrieve(x: Option<&str>) -> Result<&str> {
        x.ok_or_else(|| anyhow!("error"))
    }

    let key = retrieve(parts.next())?.trim();
    let value = retrieve(parts.next())?.trim();

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, &key[1..]),
        Some('@') => (KeyValType::Body, &key[1..]),
        Some(v) if v.is_alphabetic() => (KeyValType::Query, key),
        _ => return Err(anyhow!("invalid key val")),
    };

    Ok(KeyVal {
        key_type,
        key: key.to_string(),
        val: value.to_string(),
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(args: Vec<KeyVal>) -> Self {
        let mut headers = vec![];
        let mut query = vec![];
        let mut body = vec![];

        for arg in args {
            match arg.key_type {
                KeyValType::Header => headers.push((arg.key, arg.val)),
                KeyValType::Query => query.push((arg.key, arg.val)),
                KeyValType::Body => body.push((arg.key, arg.val)),
            }
        }

        Self {
            headers,
            query,
            body,
        }
    }
}
