use std::time::Duration;

pub enum EventLoopMsg {
    Tick,
    UserEvent(Box<dyn std::any::Any + Send>),
    Quit,
}

pub trait EventLoop {
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn quit(&mut self);
    fn post_event(&mut self, msg: EventLoopMsg);
}

pub struct SimpleEventLoop {
    running: bool,
    tick_interval: Option<Duration>,
    on_tick: Option<Box<dyn FnMut()>>,
}

impl SimpleEventLoop {
    pub fn new() -> Self {
        SimpleEventLoop {
            running: false,
            tick_interval: None,
            on_tick: None,
        }
    }

    pub fn with_tick_interval(mut self, interval: Duration) -> Self {
        self.tick_interval = Some(interval);
        self
    }

    pub fn on_tick<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.on_tick = Some(Box::new(callback));
        self
    }
}

impl EventLoop for SimpleEventLoop {
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = true;
        while self.running {
            if let Some(ref mut tick) = self.on_tick {
                tick();
            }
            if let Some(interval) = self.tick_interval {
                std::thread::sleep(interval);
            } else {
                break;
            }
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn post_event(&mut self, _msg: EventLoopMsg) {
        // Simple implementation — in production, use channel-based IPC
    }
}
