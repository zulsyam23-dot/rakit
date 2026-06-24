use std::time::Instant;

pub type TestFn = Box<dyn Fn() -> Result<(), String>>;

pub struct RakitTest {
    pub name: String,
    pub location: String,
    pub fn_ptr: TestFn,
    pub result: Option<TestResult>,
}

pub enum TestResult {
    Passed,
    Failed { message: String, stack: Option<String> },
    Error { message: String },
}

pub struct TestRunner {
    pub tests: Vec<RakitTest>,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: f64,
}

impl TestRunner {
    pub fn new() -> Self {
        TestRunner {
            tests: Vec::new(),
            passed: 0,
            failed: 0,
            duration_ms: 0.0,
        }
    }

    pub fn register(&mut self, name: String, location: String, test_fn: TestFn) {
        self.tests.push(RakitTest {
            name,
            location,
            fn_ptr: test_fn,
            result: None,
        });
    }

    pub fn run(&mut self) {
        let start = Instant::now();

        for test in &self.tests {
            match (test.fn_ptr)() {
                Ok(()) => {
                    self.passed += 1;
                    println!("  [LULUS] {} — lulus", test.name);
                }
                Err(msg) => {
                    self.failed += 1;
                    println!("  [GAGAL] {} — GAGAL: {}", test.name, msg);
                }
            }
        }

        self.duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        println!();
        println!("╔══════════════════════════════════╗");
        println!("║  Test Summary                    ║");
        println!("╠══════════════════════════════════╣");
        println!("║  Total:  {:>5} tests            ║", self.tests.len());
        println!("║  Passed: {:>5}                  ║", self.passed);
        println!("║  Failed: {:>5}                  ║", self.failed);
        println!("║  Time:   {:>5.1}ms              ║", self.duration_ms);
        println!("╚══════════════════════════════════╝");
    }
}

pub fn assert_eq<T: PartialEq + std::fmt::Debug>(
    actual: T,
    expected: T,
    msg: &str,
) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{}: expected {:?}, got {:?}", msg, expected, actual))
    }
}

pub fn assert_true(actual: bool, msg: &str) -> Result<(), String> {
    if actual {
        Ok(())
    } else {
        Err(format!("{}: expected true, got false", msg))
    }
}

pub fn assert_false(actual: bool, msg: &str) -> Result<(), String> {
    if !actual {
        Ok(())
    } else {
        Err(format!("{}: expected false, got true", msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_runner_passed_failed() {
        let mut runner = TestRunner::new();
        runner.register(
            "test1".into(),
            "test.rakit:1".into(),
            Box::new(|| Ok(())),
        );
        runner.register(
            "test2".into(),
            "test.rakit:2".into(),
            Box::new(|| Err("sengaja gagal".into())),
        );
        runner.run();
        assert_eq!(runner.passed, 1);
        assert_eq!(runner.failed, 1);
    }

    #[test]
    fn test_assert_eq_ok() {
        assert!(assert_eq(42, 42, "nilai sama").is_ok());
    }

    #[test]
    fn test_assert_eq_err() {
        assert!(assert_eq(42, 0, "nilai beda").is_err());
    }

    #[test]
    fn test_assert_true_ok() {
        assert!(assert_true(true, "benar").is_ok());
    }

    #[test]
    fn test_assert_true_err() {
        assert!(assert_true(false, "salah").is_err());
    }

    #[test]
    fn test_assert_false_ok() {
        assert!(assert_false(false, "salah").is_ok());
    }

    #[test]
    fn test_assert_false_err() {
        assert!(assert_false(true, "benar").is_err());
    }
}
