use errors::*;

use rustc_serialize::Decodable;
use std::fs::File;
use std::io::prelude::*;
use toml::{Parser, Value};
use toml::decode as toml_decode;

#[derive(Debug, Clone, RustcDecodable)]
pub struct Options {
    pub template: String,
    pub threads: usize,
}

// This is kind of preparation for other config format
// still not sure I'll need it in sucha a small program
#[derive(Debug, Clone)]
pub enum MainConfig {
    Toml(TomlConfig),
}

impl MainConfig {
    pub fn from_file(filename: &str) -> Result<MainConfig> {
        if filename.ends_with("toml") {
            TomlConfig::from_file(filename).map(MainConfig::Toml)
        } else {
            Err(ErrorKind::ConfigError("unknown extension").into())
        }
    }
    pub fn decode_tag<C: Decodable + Default>(&self, tag: &str) -> Result<C> {
        match *self {
            MainConfig::Toml(ref c) => c.decode_tag(tag),
        }
    }
    pub fn options(&self) -> &Options {
        match *self {
            MainConfig::Toml(ref c) => &c.options,
        }
    }
    pub fn watchers(&self) -> &Value {
        match *self {
            MainConfig::Toml(ref c) => &c.watchers,
        }
    }
}
#[derive(Debug, Clone)]
pub struct TomlConfig {
    options: Options,
    watchers: Value,
}

impl TomlConfig {
    pub fn from_file(filename: &str) -> Result<TomlConfig> {
        let mut config_str = String::new();
        File::open(filename).and_then(|mut f| f.read_to_string(&mut config_str))?;
        TomlConfig::from_str(&config_str)
    }

    pub fn from_str(config_str: &str) -> Result<TomlConfig> {
        let mut parser = Parser::new(&config_str);
        let table = Value::Table(parser.parse().ok_or(ErrorKind::ConfigError("parsing config"))?);
        let options = toml_decode(table.clone()).ok_or(ErrorKind::ConfigError("parsing options"))?;
        Ok(TomlConfig {
            options: options,
            watchers: table,
        })
    }

    // Parse watcher config by tag string given
    pub fn decode_tag<C: Decodable + Default>(&self, tag: &str) -> Result<C> {
        let watchers =
            &self.watchers.as_table().ok_or(ErrorKind::ConfigError("table expected"))?["watcher"];
        let watchers = watchers.as_table().ok_or(ErrorKind::ConfigError("table expected"))?;
        let watchers = watchers.get(tag).ok_or(ErrorKind::ConfigError("tag not found"))?;
        toml_decode(watchers.clone()).ok_or(ErrorKind::ConfigError("parsing watcher config").into())
    }
}
