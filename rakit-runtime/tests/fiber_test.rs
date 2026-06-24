use rakit_runtime::{FiberRoot, Scheduler};

#[test]
fn test_fiber_root_creation() {
    let mut root = FiberRoot::new();
    let id = root.create_fiber("div", None, 0);
    assert_eq!(id, 1);
    assert_eq!(root.root_id, 1);
    assert!(root.get_fiber(id).is_some());
}

#[test]
fn test_fiber_tree_structure() {
    let mut root = FiberRoot::new();
    let parent = root.create_fiber("div", None, 0);
    let child1 = root.create_fiber("p", Some(parent), 1);
    let child2 = root.create_fiber("span", Some(parent), 1);

    let fiber = root.get_fiber(parent).unwrap();
    assert_eq!(fiber.children.len(), 2);
    assert!(fiber.children.contains(&child1));
    assert!(fiber.children.contains(&child2));
}

#[test]
fn test_fiber_props() {
    let mut root = FiberRoot::new();
    let id = root.create_fiber("button", None, 0);

    if let Some(fiber) = root.get_fiber_mut(id) {
        fiber.set_props(vec![
            ("class".to_string(), "btn".to_string()),
            ("id".to_string(), "submit".to_string()),
        ]);
    }

    let fiber = root.get_fiber(id).unwrap();
    assert_eq!(fiber.props.get("class"), Some(&"btn".to_string()));
    assert_eq!(fiber.props.get("id"), Some(&"submit".to_string()));
}

#[test]
fn test_fiber_dirty_flag() {
    let mut root = FiberRoot::new();
    let id = root.create_fiber("div", None, 0);

    let fiber = root.get_fiber(id).unwrap();
    assert!(fiber.dirty);

    if let Some(fiber) = root.get_fiber_mut(id) {
        fiber.dirty = false;
    }

    root.mark_dirty(id);
    let fiber = root.get_fiber(id).unwrap();
    assert!(fiber.dirty);
}

#[test]
fn test_fiber_child_parent_relationship() {
    let mut root = FiberRoot::new();
    let parent = root.create_fiber("ul", None, 0);
    let child = root.create_fiber("li", Some(parent), 1);

    let fiber = root.get_fiber(child).unwrap();
    assert_eq!(fiber.parent, Some(parent));
}

#[test]
fn test_scheduler_create_root() {
    let mut scheduler = Scheduler::new();
    let id = scheduler.create_root("app");
    assert_eq!(id, 1);
}

#[test]
fn test_scheduler_append_child() {
    let mut scheduler = Scheduler::new();
    let parent = scheduler.create_root("div");
    let child = scheduler.append_child(parent, "p");

    let child_fiber = scheduler.root.get_fiber(child).unwrap();
    assert_eq!(child_fiber.parent, Some(parent));
    assert_eq!(child_fiber.depth, 1);
}

#[test]
fn test_scheduler_schedule_update() {
    let mut scheduler = Scheduler::new();
    let id = scheduler.create_root("div");

    scheduler.schedule_update(id);
    let fiber = scheduler.root.get_fiber(id).unwrap();
    assert!(!fiber.dirty);
}

#[test]
fn test_fiber_depth() {
    let mut root = FiberRoot::new();
    let root_fiber = root.create_fiber("div", None, 0);
    let child = root.create_fiber("p", Some(root_fiber), 1);
    let grandchild = root.create_fiber("span", Some(child), 2);

    assert_eq!(root.get_fiber(root_fiber).unwrap().depth, 0);
    assert_eq!(root.get_fiber(child).unwrap().depth, 1);
    assert_eq!(root.get_fiber(grandchild).unwrap().depth, 2);
}

#[test]
fn test_scheduler_reconcile() {
    let mut scheduler = Scheduler::new();
    let id = scheduler.create_root("div");

    scheduler.schedule_update(id);
    scheduler.reconcile();

    assert!(scheduler.work_queue.is_empty());
}
