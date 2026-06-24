use std::cell::RefCell;
use std::rc::Rc;
use rakit_runtime::{use_state, use_effect, use_memo, use_ref};
use rakit_runtime::{HookContext, RakitValue};

#[test]
fn test_hook_context_new() {
    let ctx = HookContext::new();
    assert_eq!(ctx.index, 0);
    assert!(ctx.hooks.is_empty());
}

#[test]
fn test_use_state_initial_value() {
    let mut ctx = HookContext::new();
    let (value, _setter) = use_state(&mut ctx, 42i32);
    assert_eq!(value, 42);
    assert_eq!(ctx.index, 1);
}

#[test]
fn test_use_state_preserves_across_rerender() {
    let mut ctx = HookContext::new();
    let (_value, setter) = use_state(&mut ctx, 0i32);
    setter(99);
    ctx.reset();
    let (value, _) = use_state(&mut ctx, 0i32);
    assert_eq!(value, 0);
    assert_eq!(ctx.index, 1);
}

#[test]
fn test_use_state_multiple_hooks() {
    let mut ctx = HookContext::new();
    let (a, _) = use_state(&mut ctx, 1i32);
    let (b, _) = use_state(&mut ctx, "hello");
    let (c, _) = use_state(&mut ctx, true);
    assert_eq!(a, 1);
    assert_eq!(b, "hello");
    assert_eq!(c, true);
    assert_eq!(ctx.index, 3);

    ctx.reset();
    let (a2, _) = use_state(&mut ctx, 99i32);
    let (b2, _) = use_state(&mut ctx, "world");
    let (c2, _) = use_state(&mut ctx, false);
    assert_eq!(a2, 1);
    assert_eq!(b2, "hello");
    assert_eq!(c2, true);
}

#[test]
fn test_use_effect_tracking() {
    let mut ctx = HookContext::new();
    let effect_ran = Rc::new(RefCell::new(false));
    let effect_ran_clone = effect_ran.clone();

    use_effect(&mut ctx, vec![1, 2, 3], move || {
        *effect_ran_clone.borrow_mut() = true;
        None
    });

    assert_eq!(*effect_ran.borrow(), true);
}

#[test]
fn test_use_effect_deps_change() {
    let mut ctx = HookContext::new();
    let run_count = Rc::new(RefCell::new(0));

    let rc = run_count.clone();
    use_effect(&mut ctx, vec![1], move || {
        *rc.borrow_mut() += 1;
        None
    });
    assert_eq!(*run_count.borrow(), 1);

    ctx.reset();
    let rc2 = run_count.clone();
    use_effect(&mut ctx, vec![1], move || {
        *rc2.borrow_mut() += 1;
        None
    });
    assert_eq!(*run_count.borrow(), 1);

    ctx.reset();
    let rc3 = run_count.clone();
    use_effect(&mut ctx, vec![2], move || {
        *rc3.borrow_mut() += 1;
        None
    });
    assert_eq!(*run_count.borrow(), 2);
}

#[test]
fn test_use_memo_caches() {
    let mut ctx = HookContext::new();
    let compute_count = Rc::new(RefCell::new(0));

    let cc = compute_count.clone();
    let result1 = use_memo(&mut ctx, vec![1, 2], move || {
        *cc.borrow_mut() += 1;
        42
    });
    assert_eq!(result1, Some(42));
    assert_eq!(*compute_count.borrow(), 1);

    ctx.reset();
    let cc2 = compute_count.clone();
    let result2 = use_memo(&mut ctx, vec![1, 2], move || {
        *cc2.borrow_mut() += 1;
        99
    });
    assert_eq!(result2, Some(42));
    assert_eq!(*compute_count.borrow(), 1);

    ctx.reset();
    let cc3 = compute_count.clone();
    let result3 = use_memo(&mut ctx, vec![3, 4], move || {
        *cc3.borrow_mut() += 1;
        99
    });
    assert_eq!(result3, Some(99));
    assert_eq!(*compute_count.borrow(), 2);
}

#[test]
fn test_use_ref_basics() {
    let mut ctx = HookContext::new();
    let r = use_ref(&mut ctx, 0i32);
    assert_eq!(r.get(), 0);

    r.set(42);
    assert_eq!(r.get(), 42);

    ctx.reset();
    let r2 = use_ref(&mut ctx, 99i32);
    assert_eq!(r2.get(), 42);
}

#[test]
fn test_rakit_value_conversions() {
    let n: RakitValue = 42.0.into();
    assert_eq!(n.as_number(), Some(42.0));

    let t: RakitValue = "hello".into();
    assert_eq!(t.as_text(), Some("hello".to_string()));

    let b: RakitValue = true.into();
    assert_eq!(b.as_bool(), Some(true));
}

#[test]
fn test_rakit_value_truthiness() {
    assert!(!RakitValue::Null.is_truthy());
    assert!(RakitValue::Bool(true).is_truthy());
    assert!(!RakitValue::Bool(false).is_truthy());
    assert!(RakitValue::Number(1.0).is_truthy());
    assert!(!RakitValue::Number(0.0).is_truthy());
    assert!(RakitValue::Text("hi".into()).is_truthy());
    assert!(!RakitValue::Text("".into()).is_truthy());
}
