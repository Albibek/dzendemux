#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate toml;
extern crate rustc_serialize;
extern crate tokio_core;
extern crate tokio_timer;
extern crate futures;
extern crate futures_cpupool;
extern crate liquid;

mod config;
mod errors;

pub mod watcher;
mod watchers {
    pub mod clock;
    pub mod file;
}

use config::MainConfig;
use std::env;
use std::path::Path;
use watcher::WatchLoop;

fn main() {
    let mut args = env::args();
    args.next().unwrap(); // Remove program name
    let config_file = match args.next() {
        Some(filename) => filename,
        None => match env::var("HOME") {
            Ok(ref home) => {
                let path = Path::new(home);
                path.join(".dzendemux.toml").to_str().unwrap().to_string()
            }
            Err(_) => "dzendemux.toml".to_string()
        }
    };

    let config = MainConfig::from_file(&config_file).unwrap();

    let mut l = WatchLoop::new(config);
    l.add_all_builtin();
    l.run().unwrap();
}
