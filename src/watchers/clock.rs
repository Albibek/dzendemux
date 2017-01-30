use chrono::Local;
use config::MainConfig;
use watcher::*;

#[derive(Debug, RustcDecodable, Clone)]
struct ClockWatcherConfig {
    format: String,
}

impl Default for ClockWatcherConfig {
    fn default() -> ClockWatcherConfig {
        ClockWatcherConfig { format: "%v %T".to_string() }
    }
}


#[derive(Default, Clone)]
pub struct ClockWatcher {
    tag: String,
    config: ClockWatcherConfig,
}

impl Watcher for ClockWatcher {
    fn configure(&mut self, main_conf: &MainConfig, tag: &str) {
        self.config = main_conf.decode_tag(tag).unwrap();
        self.tag = tag.to_string();
    }

    fn into_action(self) -> Box<Fn() -> Tagged + Send> {
        let z = move || -> Tagged {
            let now = format!("{}", Local::now().format(&self.config.format));
            Tagged {
                tag: self.tag.clone(),
                data: now,
            }
        };

        Box::new(z)
    }
}
