use config::MainConfig;
use errors::*;

use futures::{Sink, Stream};
use futures::future::{empty, ok};
use futures::sync::mpsc::{channel, Sender};
use futures_cpupool::CpuPool;

use liquid::{parse, LiquidOptions, Context, Value, Renderable};

use std::clone::Clone;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio_core::reactor::{Handle, Core};
use tokio_timer::{Timer, wheel};
use watchers::clock::ClockWatcher;
use watchers::file::FileWatcher;

#[derive(Debug)]
pub struct Tagged {
    pub data: String,
    pub tag: String,
}

pub struct WatchLoop {
    config: MainConfig,
    core: Core,
    handle: Handle,
    timer: Timer,
    sender: Sender<Tagged>,
    builders: HashMap<&'static str, Box<Builder>>,
    pool: CpuPool,
}

impl WatchLoop {
    pub fn new(config: MainConfig) -> Self {
        let options = config.options().clone();
        let core = Core::new().unwrap();
        let handle = core.handle();

        // TODO think about timer settings, currently just accept them
        let timer = wheel().build();

        // 16 messages will be enough for everyone :)
        let (tx, rx) = channel::<Tagged>(16);
        let pool = CpuPool::new(options.threads);

        let template = parse(&options.template, LiquidOptions::default()).unwrap();
        let mut context = Context::new();

        let status = rx.and_then(move |e| {
            context.set_val(&e.tag, Value::Str(e.data));
            let render = template.render(&mut context).unwrap();

            println!("{}", render.unwrap_or("None".to_string()));
            ok(())
        });

        let status = status.for_each(|_| Ok(()));
        handle.spawn(status);

        WatchLoop {
            config: config,
            core: core,
            handle: handle,
            timer: timer,
            sender: tx,
            builders: HashMap::new(),
            pool: pool,
        }
    }

    pub fn add_all_builtin(&mut self) {
        let clock = ClockWatcher::default();
        self.builders.insert("clock", Box::new(clock) as Box<Builder>);

        let file = FileWatcher::default();
        self.builders.insert("file", Box::new(file) as Box<Builder>);
    }

    pub fn add_builder(&mut self, wtype: &'static str, b: Box<Builder>) {
        self.builders.insert(wtype, b);
    }

    pub fn run(&mut self) -> Result<()> {
        let watchers = &self.config
            .watchers()
            .as_table()
            .ok_or(ErrorKind::ConfigError("table expected"))?["watcher"];
        let watchers = watchers.as_table().ok_or(ErrorKind::ConfigError("table expected"))?;
        for (tag, table) in watchers {
            let table = table.as_table().ok_or(ErrorKind::ConfigError("table expected"))?;
            // TODO: warning when tick not found
            let tick = table.get("tick").ok_or(ErrorKind::ConfigError("tick option not found "))?;
            let tick = tick.as_integer().ok_or(ErrorKind::ConfigError("integer expected"))?;
            let tick = Duration::from_millis(tick as u64);
            let wtype = table.get("type").ok_or(ErrorKind::ConfigError("type option not found"))?;
            let wtype = wtype.as_str().ok_or(ErrorKind::ConfigError("type must be string"))?;
            // TODO log if default tick is used
            let mut builder = self.builders
                .get_mut(wtype)
                .ok_or(ErrorKind::ConfigError("type not found"))?;
            builder.configure(&self.config, tag);
            builder.build(tick,
                          &self.timer,
                          self.sender.clone(),
                          &self.handle,
                          &self.pool);
        }

        self.core.run(empty::<(), ()>()).map_err(|()| ErrorKind::CoreRunError.into() )
    }
}

pub trait Watcher {
    fn configure(&mut self, main_conf: &MainConfig, tag: &str);
    fn into_action(self) -> Box<Fn() -> Tagged + Send>;
    fn is_blocking(&self) -> bool {
        false
    }
}

pub trait Builder: Watcher {
    fn build(&self, Duration, &Timer, Sender<Tagged>, &Handle, &CpuPool);
}

impl<T: Watcher + Send + Clone> Builder for T {
    fn build(&self,
             interval: Duration,
             timer: &Timer,
             mut tx: Sender<Tagged>,
             handle: &Handle,
             pool: &CpuPool) {

        let a = self.clone().into_action();

        let now = Instant::now() + Duration::from_secs(1);
        let w = timer.interval_at(now, interval)
            .map_err(move |_| ()); // Rmove TimerErrors. TODO: deal with them


        // There is magic in this string
        // FnMut is unsized because we got it from the box
        // but map somehow requires it to be sized
        // so we have to work this around it by running it inside another move closure
        // this though allows us to skip passing () to watcher actions
        // TODO:
        // change start_send to send+flush+timeout+ (no send + error on timeout)
        // to ensure queue is not taking all the memory when racing with timers

        let w = w.map(move |_| tx.start_send(a()));
        let w = w.for_each(|_| Ok(()));
        if !self.is_blocking() {
            // eliminate errors
            // TODO insert slog logging send errors here
            handle.spawn(w);
        } else {
            let w = pool.spawn(w);
            handle.spawn(w);
        }
    }
}
