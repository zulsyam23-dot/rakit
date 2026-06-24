use std::rc::Rc;

type StreamCallback<T> = Rc<dyn Fn(T)>;

pub struct DataStream<T: Clone + 'static> {
    subscribers: Vec<StreamCallback<T>>,
    latest: Option<T>,
}

impl<T: Clone + 'static> DataStream<T> {
    pub fn new() -> Self {
        DataStream {
            subscribers: Vec::new(),
            latest: None,
        }
    }

    pub fn subscribe(&mut self, callback: StreamCallback<T>) {
        if let Some(ref value) = self.latest {
            callback(value.clone());
        }
        self.subscribers.push(callback);
    }

    pub fn emit(&mut self, value: T) {
        self.latest = Some(value.clone());
        for callback in &self.subscribers {
            callback(value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn test_data_stream_emit_and_receive() {
        let mut stream: DataStream<i32> = DataStream::new();
        let received = Rc::new(RefCell::new(Vec::new()));
        let r = received.clone();
        stream.subscribe(Rc::new(move |v| r.borrow_mut().push(v)));
        stream.emit(42);
        assert_eq!(received.borrow().len(), 1);
        assert_eq!(received.borrow()[0], 42);
    }

    #[test]
    fn test_data_stream_latest_on_subscribe() {
        let mut stream: DataStream<i32> = DataStream::new();
        stream.emit(100);
        let received = Rc::new(RefCell::new(0));
        let r = received.clone();
        stream.subscribe(Rc::new(move |v| *r.borrow_mut() = v));
        assert_eq!(*received.borrow(), 100);
    }
}
