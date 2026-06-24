use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub pesan: String,
    pub komponen: String,
    pub stack: Option<String>,
}

thread_local! {
    pub static BOUNDARY_STACK: RefCell<Vec<BoundaryEntry>> = RefCell::new(Vec::new());
    pub static LAST_ERROR: RefCell<Option<ErrorInfo>> = RefCell::new(None);
}

pub struct BoundaryEntry {
    pub fiber_id: u64,
    pub fallback: Box<dyn Fn(ErrorInfo) -> String>,
}

pub fn register_boundary<F: Fn(ErrorInfo) -> String + 'static>(
    fiber_id: u64,
    fallback: F,
) {
    BOUNDARY_STACK.with(|stack| {
        stack.borrow_mut().push(BoundaryEntry {
            fiber_id,
            fallback: Box::new(fallback),
        });
    });
}

pub fn unregister_boundary(fiber_id: u64) {
    BOUNDARY_STACK.with(|stack| {
        stack.borrow_mut().retain(|e| e.fiber_id != fiber_id);
    });
}

pub fn catch_error(error: ErrorInfo) -> Option<String> {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error.clone());
    });

    BOUNDARY_STACK.with(|stack| {
        let stack = stack.borrow();
        if let Some(boundary) = stack.last() {
            Some((boundary.fallback)(error))
        } else {
            None
        }
    })
}

pub fn try_render<T>(
    fiber_id: u64,
    component_name: &str,
    render_fn: impl FnOnce() -> T,
    fallback_fn: impl Fn(ErrorInfo) -> T,
) -> T {
    register_boundary(fiber_id, |_err| String::new());

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| render_fn()));

    unregister_boundary(fiber_id);

    match result {
        Ok(value) => value,
        Err(panic_info) => {
            let error = ErrorInfo {
                pesan: extract_panic_message(&panic_info),
                komponen: component_name.to_string(),
                stack: Some(capture_stack_trace()),
            };
            (fallback_fn)(error)
        }
    }
}

fn extract_panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "Terjadi error yang tidak diketahui".to_string()
    }
}

fn capture_stack_trace() -> String {
    "Stack trace tidak tersedia".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_boundary_catches_panic() {
        let fiber_id = 1;
        let result = try_render(
            fiber_id,
            "TestComponent",
            || {
                panic!("Sengaja error!");
            },
            |err: ErrorInfo| -> String {
                format!("Error: {}", err.pesan)
            },
        );

        assert!(result.contains("Sengaja error!"));
    }

    #[test]
    fn test_error_boundary_no_error() {
        let fiber_id = 2;
        let result = try_render(
            fiber_id,
            "GoodComponent",
            || -> String { "Sukses".to_string() },
            |_: ErrorInfo| -> String { "Error".to_string() },
        );

        assert_eq!(result, "Sukses");
    }

    #[test]
    fn test_catch_error_returns_fallback() {
        let fiber_id = 3;
        register_boundary(fiber_id, |err: ErrorInfo| {
            format!("Boundary: {}", err.pesan)
        });

        let error = ErrorInfo {
            pesan: "Test error".to_string(),
            komponen: "Test".to_string(),
            stack: None,
        };

        let result = catch_error(error);
        assert!(result.is_some());
        unregister_boundary(fiber_id);
    }
}
