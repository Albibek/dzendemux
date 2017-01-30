use chrono::Local;
use config::MainConfig;
use watcher::*;


#[derive(Default, Clone)]
pub struct ClockWatcher {
    tag: String,
}

impl Watcher for ClockWatcher {
    fn configure(&mut self, _: &MainConfig, tag: &str) {
        self.tag = tag.to_string();
    }

    fn into_action(self) -> Box<Fn() -> Tagged + Send> {
        let z = move || -> Tagged {
            let now = Local::now();
            Tagged {
                tag: self.tag.clone(),
                data: now.to_rfc2822(),
            }
        };

        Box::new(z)
    }
}
