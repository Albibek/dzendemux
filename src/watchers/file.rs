use config::MainConfig;

use std::fs::File;
use std::io::prelude::*;
use watcher::*;

#[derive(Debug, RustcDecodable, Clone)]
struct FileWatcherConfig {
    filename: String,
}

#[derive(Debug, Clone, Default)]
pub struct FileWatcher {
    tag: String,
    config: FileWatcherConfig,
}

impl Default for FileWatcherConfig {
    fn default() -> FileWatcherConfig {
        FileWatcherConfig { filename: "/nonexistent".to_string() }
    }
}

impl Watcher for FileWatcher {
    fn configure(&mut self, main_conf: &MainConfig, tag: &str) {
        self.config = main_conf.decode_tag(tag).unwrap();
        self.tag = tag.to_string();
    }

    fn into_action(self) -> Box<Fn() -> Tagged + Send> {
        let z = move || -> Tagged {
            let data = self.read_file(&self.config.filename, false);
            Tagged {
                tag: self.tag.clone(),
                data: data.trim().into(),
            }
        };

        Box::new(z)
    }
    fn is_blocking(&self) -> bool {
        true
    }
}

impl FileWatcher {
    fn read_file(&self, filename: &str, strip: bool) -> String {
        let mut content = String::new();
        // file lifetime limitation
        {
            let file = File::open(filename);
            if file.is_ok() {
                file.unwrap().read_to_string(&mut content).unwrap_or(0);
            } else {
                content = format!("<no file: {}>", filename);
            }
        }
        if strip {
            content = content.trim().to_string();
        }
        content
    }
}
