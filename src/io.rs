use clap::ValueEnum;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use mustache::Data;
use serde_json::Value;

use crate::CliArgs;

#[derive(ValueEnum, Clone, Default, Debug)]
pub enum ValuesFormat {
    #[default]
    Json,
    Yaml,
    Toml,
}

impl From<&PathBuf> for ValuesFormat {
    fn from(value: &PathBuf) -> Self {
        match value.extension().and_then(|s| s.to_str()).unwrap_or("") {
            "json" => Self::Json,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            _ => Self::default(),
        }
    }
}

pub fn parse_values(cli_args: &CliArgs) -> Result<mustache::Data> {
    let file_path = cli_args.file.clone();
    let mut source: Box<dyn Read> = if let Some(path) = &file_path {
        let file = File::open(path.clone())?;
        Box::new(BufReader::new(file))
    } else {
        Box::new(std::io::stdin().lock())
    };

    let json_values = match file_path.map(|p| (&p).into()).unwrap_or_default() {
        ValuesFormat::Json => serde_json::from_reader(source).context("failed to parse json"),
        ValuesFormat::Yaml => serde_yaml::from_reader(source).context("failed to parse yaml"),
        ValuesFormat::Toml => {
            let mut buf = String::new();
            source.read_to_string(&mut buf)?;
            toml::from_str(&buf).context("failed to parse toml")
        }
    };
    serde_json_to_mustache(json_values?)
}

pub fn expand_path(path: &Path) -> Result<PathBuf> {
    let expanded = shellexpand::full(path.to_str().expect("template path was not valid UTF-8"))?;
    Ok(PathBuf::from(expanded.into_owned()))
}

pub fn serde_json_to_mustache(value: Value) -> anyhow::Result<Data> {
    Ok(match value {
        Value::Object(x) => Data::Map(
            x.into_iter()
                .map(|(k, v)| serde_json_to_mustache(v).map(|d| (k, d)))
                .collect::<Result<HashMap<String, Data>, _>>()?,
        ),
        Value::Array(x) => Data::Vec(
            x.into_iter()
                .map(serde_json_to_mustache)
                .collect::<Result<Vec<Data>, _>>()?,
        ),
        Value::String(x) => Data::String(x),
        Value::Number(x) => Data::String(x.to_string()),
        Value::Bool(x) => Data::Bool(x),
        Value::Null => Data::Null,
    })
}
