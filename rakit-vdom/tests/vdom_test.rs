use rakit_vdom::node::*;
use rakit_vdom::h::{h, text, fragment, h_with_key, HChild};
use rakit_vdom::render::ssr::SsrRenderer;
use rakit_vdom::render::native::NativeRenderer;
use rakit_vdom::diff::{diff, Patch};
use rakit_vdom::patch::native::NativePatchApplicator;
use rakit_vdom::utils::{get_node_key, props_changed, escape_html};
use std::collections::HashMap;

// ─── NODE TESTS ───

#[test]
fn test_element_creation() {
    let node = VDomNode::element("div", vec![("class", "container")], vec![]);
    assert_eq!(node.tag(), Some("div"));
    assert!(node.children().is_empty());
}

#[test]
fn test_text_node() {
    let node = VDomNode::text("hello");
    match &node {
        VDomNode::Text(t) => assert_eq!(t.value, "hello"),
        _ => panic!("expected Text"),
    }
}

#[test]
fn test_fragment_creation() {
    let children = vec![VDomNode::text("a"), VDomNode::text("b")];
    let frag = VDomNode::fragment(children);
    assert!(frag.is_fragment());
    assert_eq!(frag.children().len(), 2);
}

#[test]
fn test_component_node() {
    let mut props = HashMap::new();
    props.insert("label".to_string(), AttrValue::String("Click".into()));
    let node = VDomNode::component("Button", props, Some("btn-1".into()));
    match &node {
        VDomNode::Component(c) => {
            assert_eq!(c.name, "Button");
            assert_eq!(c.key.as_deref(), Some("btn-1"));
        }
        _ => panic!("expected Component"),
    }
}

#[test]
fn test_empty_node() {
    let node = VDomNode::empty();
    assert!(node.is_empty());
}

#[test]
fn test_key_support() {
    let with_key = VDomNode::Element(ElementNode {
        tag: "li".into(),
        attrs: Default::default(),
        events: Default::default(),
        children: vec![],
        key: Some("item-1".into()),
    });
    assert_eq!(with_key.key(), Some("item-1"));

    let without_key = VDomNode::element("li", vec![], vec![]);
    assert_eq!(without_key.key(), None);
}

#[test]
fn test_is_same_type() {
    let div1 = VDomNode::element("div", vec![], vec![]);
    let div2 = VDomNode::element("div", vec![("class", "x")], vec![]);
    let span = VDomNode::element("span", vec![], vec![]);
    let text1 = VDomNode::text("a");
    let text2 = VDomNode::text("b");

    assert!(div1.is_same_type(&div2));
    assert!(!div1.is_same_type(&span));
    assert!(text1.is_same_type(&text2));
    assert!(!div1.is_same_type(&text1));
    assert!(VDomNode::empty().is_same_type(&VDomNode::empty()));
}

#[test]
fn test_flatten_fragments() {
    let inner = VDomNode::fragment(vec![VDomNode::text("a"), VDomNode::text("b")]);
    let outer = VDomNode::fragment(vec![inner, VDomNode::text("c")]);
    let flat = outer.flatten_fragments();
    assert_eq!(flat.len(), 3);
}

#[test]
fn test_element_with_attrs_api() {
    let mut attrs = HashMap::new();
    attrs.insert("id".into(), AttrValue::String("main".into()));
    let mut events = HashMap::new();
    events.insert("click".into(), 1);

    let node = VDomNode::element_with_attrs("div", attrs, events, vec![], None);
    match &node {
        VDomNode::Element(e) => {
            assert_eq!(e.attrs.get("id").and_then(|v| v.as_string()), Some("main"));
            assert_eq!(e.events.get(&EventType::Click), Some(&1));
        }
        _ => panic!("expected Element"),
    }
}

// ─── H() FUNCTION TESTS ───

#[test]
fn test_h_basic() {
    let node = h("button", vec![("class", "btn")], vec![HChild::Text("Klik".into())]);
    match &node {
        VDomNode::Element(e) => {
            assert_eq!(e.tag, "button");
            assert_eq!(e.attrs.get("class").and_then(|v| v.as_string()), Some("btn"));
        }
        _ => panic!("expected Element"),
    }
}

#[test]
fn test_h_nested() {
    let node = h("div", vec![], vec![
        HChild::Node(h("h1", vec![], vec![HChild::Text("Judul".into())])),
        HChild::Node(h("p", vec![], vec![HChild::Text("Konten".into())])),
    ]);
    assert_eq!(node.children().len(), 2);
}

#[test]
fn test_h_with_key() {
    let node = h_with_key("li", "item-1", vec![], vec![HChild::Text("Item".into())]);
    assert_eq!(node.key(), Some("item-1"));
}

#[test]
fn test_fragment_helper() {
    let frag = fragment(vec![text("a"), text("b")]);
    assert_eq!(frag.children().len(), 2);
}

#[test]
fn test_text_helper() {
    let t = text("hello");
    match &t {
        VDomNode::Text(tn) => assert_eq!(tn.value, "hello"),
        _ => panic!("expected Text"),
    }
}

// ─── SSR TESTS ───

#[test]
fn test_ssr_simple_element() {
    let node = VDomNode::element("div", vec![("class", "app")], vec![VDomNode::text("Halo")]);
    let mut renderer = SsrRenderer::new(false);
    let html = renderer.render(&node);
    assert_eq!(html, r#"<div class="app">Halo</div>"#);
}

#[test]
fn test_ssr_nested_elements() {
    let node = VDomNode::element("div", vec![], vec![
        VDomNode::element("h1", vec![], vec![VDomNode::text("Judul")]),
        VDomNode::element("p", vec![], vec![VDomNode::text("Paragraf")]),
    ]);
    let mut renderer = SsrRenderer::new(true);
    let html = renderer.render(&node);
    assert!(html.contains("Judul"));
    assert!(html.contains("Paragraf"));
    assert!(html.contains("<div>"));
    assert!(html.contains("</div>"));
}

#[test]
fn test_ssr_self_closing() {
    let mut renderer = SsrRenderer::new(false);
    let html = renderer.render(&VDomNode::element("br", vec![], vec![]));
    assert_eq!(html, "<br />");
}

#[test]
fn test_ssr_empty_element() {
    let mut renderer = SsrRenderer::new(false);
    let html = renderer.render(&VDomNode::element("div", vec![], vec![]));
    assert_eq!(html, "<div></div>");
}

#[test]
fn test_ssr_html_escaping() {
    let mut renderer = SsrRenderer::new(false);
    let html = renderer.render(&VDomNode::text("<script>alert('x')</script>"));
    assert_eq!(html, "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;");
}

#[test]
fn test_ssr_fragment() {
    let mut renderer = SsrRenderer::new(false);
    let frag = VDomNode::fragment(vec![VDomNode::text("A"), VDomNode::text("B")]);
    let html = renderer.render(&frag);
    assert_eq!(html, "AB");
}

#[test]
fn test_ssr_component() {
    let mut renderer = SsrRenderer::new(false);
    let comp = VDomNode::component("Counter", HashMap::new(), None);
    let html = renderer.render(&comp);
    assert!(html.contains("Counter"));
}

// ─── DIFF TESTS ───

#[test]
fn test_diff_text_changed() {
    let old = VDomNode::text("Halo");
    let new = VDomNode::text("Dunia");
    let result = diff(Some(&old), &new);
    assert_eq!(result.patches.len(), 1);
    match &result.patches[0] {
        Patch::SetText { text, .. } => assert_eq!(text, "Dunia"),
        _ => panic!("expected SetText"),
    }
}

#[test]
fn test_diff_text_unchanged() {
    let old = VDomNode::text("hello");
    let new = VDomNode::text("hello");
    let result = diff(Some(&old), &new);
    assert_eq!(result.patches.len(), 0);
}

#[test]
fn test_diff_element_replace_on_tag_change() {
    let old = VDomNode::element("div", vec![], vec![]);
    let new = VDomNode::element("span", vec![], vec![]);
    let result = diff(Some(&old), &new);
    match &result.patches[0] {
        Patch::Replace { new_node, .. } => assert_eq!(new_node.tag(), Some("span")),
        _ => panic!("expected Replace"),
    }
}

#[test]
fn test_diff_attr_changed() {
    let old = VDomNode::element("div", vec![("class", "old")], vec![]);
    let new = VDomNode::element("div", vec![("class", "new")], vec![]);
    let result = diff(Some(&old), &new);
    assert_eq!(result.patches.len(), 1);
    match &result.patches[0] {
        Patch::SetAttr { name, value, .. } => {
            assert_eq!(name, "class");
            assert_eq!(value, &AttrValue::String("new".into()));
        }
        _ => panic!("expected SetAttr"),
    }
}

#[test]
fn test_diff_attr_removed() {
    let old = VDomNode::element("div", vec![("class", "old"), ("id", "x")], vec![]);
    let new = VDomNode::element("div", vec![("class", "new")], vec![]);
    let result = diff(Some(&old), &new);
    let has_remove = result.patches.iter().any(|p| matches!(p, Patch::RemoveAttr { .. }));
    assert!(has_remove);
}

#[test]
fn test_diff_attr_added() {
    let old = VDomNode::element("div", vec![], vec![]);
    let new = VDomNode::element("div", vec![("id", "main")], vec![]);
    let result = diff(Some(&old), &new);
    let has_set = result.patches.iter().any(|p| matches!(p, Patch::SetAttr { .. }));
    assert!(has_set);
}

#[test]
fn test_diff_empty_to_element() {
    let old = VDomNode::Empty;
    let new = VDomNode::element("div", vec![], vec![VDomNode::text("Baru")]);
    let result = diff(Some(&old), &new);
    match &result.patches[0] {
        Patch::Replace { new_node, .. } => assert_eq!(new_node.tag(), Some("div")),
        _ => {}
    }
}

#[test]
fn test_diff_insert_new_child() {
    let old = VDomNode::element("ul", vec![], vec![VDomNode::element("li", vec![], vec![])]);
    let new = VDomNode::element("ul", vec![], vec![
        VDomNode::element("li", vec![], vec![]),
        VDomNode::element("li", vec![], vec![]),
    ]);
    let result = diff(Some(&old), &new);
    let has_insert = result.patches.iter().any(|p| matches!(p, Patch::Insert { .. }));
    assert!(has_insert);
}

#[test]
fn test_diff_remove_child() {
    let old = VDomNode::element("ul", vec![], vec![
        VDomNode::element("li", vec![], vec![]),
        VDomNode::element("li", vec![], vec![]),
    ]);
    let new = VDomNode::element("ul", vec![], vec![VDomNode::element("li", vec![], vec![])]);
    let result = diff(Some(&old), &new);
    let has_remove = result.patches.iter().any(|p| matches!(p, Patch::Remove { .. }));
    assert!(has_remove);
}

#[test]
fn test_diff_attrs_unchanged() {
    let old = VDomNode::element("div", vec![("class", "box")], vec![]);
    let new = VDomNode::element("div", vec![("class", "box")], vec![]);
    let result = diff(Some(&old), &new);
    assert_eq!(result.patches.len(), 0);
}

#[test]
fn test_diff_nested_element() {
    let old = VDomNode::element("div", vec![], vec![
        VDomNode::element("p", vec![("id", "a")], vec![VDomNode::text("old")]),
    ]);
    let new = VDomNode::element("div", vec![], vec![
        VDomNode::element("p", vec![("id", "a")], vec![VDomNode::text("new")]),
    ]);
    let result = diff(Some(&old), &new);
    assert!(!result.patches.is_empty());
    let has_set_text = result.patches.iter().any(|p| matches!(p, Patch::SetText { .. }));
    assert!(has_set_text);
}

// ─── KEYED RECONCILIATION TESTS ───

fn keyed_elem(key: &str) -> VDomNode {
    VDomNode::Element(ElementNode {
        tag: "div".into(),
        attrs: Default::default(),
        events: Default::default(),
        children: vec![],
        key: Some(key.into()),
    })
}

#[test]
fn test_keyed_reorder_detected() {
    let old_children = vec![keyed_elem("a"), keyed_elem("b"), keyed_elem("c")];
    let new_children = vec![keyed_elem("c"), keyed_elem("a"), keyed_elem("b")];

    let old = VDomNode::element("ul", vec![], old_children);
    let new = VDomNode::element("ul", vec![], new_children);
    let result = diff(Some(&old), &new);

    let has_move = result.patches.iter().any(|p| matches!(p, Patch::Move { .. }));
    assert!(has_move, "Should detect move with keys");
}

#[test]
fn test_keyed_reuse_no_replace() {
    let old_children = vec![keyed_elem("a"), keyed_elem("b")];
    let new_children = vec![keyed_elem("b"), keyed_elem("a")];

    let old = VDomNode::element("ul", vec![], old_children);
    let new = VDomNode::element("ul", vec![], new_children);
    let result = diff(Some(&old), &new);

    let has_replace = result.patches.iter().any(|p| matches!(p, Patch::Replace { .. }));
    assert!(!has_replace, "Keyed elements should not be replaced");
}

// ─── STATS TESTS ───

#[test]
fn test_stats_text_change() {
    let old = VDomNode::text("old");
    let new = VDomNode::text("new");
    let result = diff(Some(&old), &new);
    assert_eq!(result.stats.text_changed, 1);
}

#[test]
fn test_stats_node_replaced() {
    let old = VDomNode::element("div", vec![], vec![]);
    let new = VDomNode::element("span", vec![], vec![]);
    let result = diff(Some(&old), &new);
    assert_eq!(result.stats.nodes_replaced, 1);
}

#[test]
fn test_stats_attrs_changed() {
    let old = VDomNode::element("div", vec![("class", "old")], vec![]);
    let new = VDomNode::element("div", vec![("class", "new")], vec![]);
    let result = diff(Some(&old), &new);
    assert_eq!(result.stats.attrs_changed, 1);
}

// ─── NATIVE RENDERER TESTS ───

#[test]
fn test_native_renderer_basic() {
    let mut renderer = NativeRenderer::new();
    let handle = renderer.render_node(&VDomNode::element("div", vec![], vec![]), &0);
    assert!(handle > 0);
}

// ─── PATCH APPLICATOR TESTS ───

#[test]
fn test_patch_applicator_new() {
    let renderer = NativeRenderer::new();
    let applicator = NativePatchApplicator::new(renderer, 1);
    assert_eq!(applicator.get_parent_handle(&[]), 1);
}

// ─── UTILS TESTS ───

#[test]
fn test_get_node_key() {
    let with_key = keyed_elem("my-key");
    assert_eq!(get_node_key(&with_key), Some("my-key"));

    let without_key = VDomNode::element("div", vec![], vec![]);
    assert_eq!(get_node_key(&without_key), None);

    let text_node = VDomNode::text("hi");
    assert_eq!(get_node_key(&text_node), None);
}

#[test]
fn test_props_changed() {
    let mut old = HashMap::new();
    old.insert("a".into(), AttrValue::String("1".into()));
    let mut new = HashMap::new();
    new.insert("a".into(), AttrValue::String("2".into()));

    assert!(props_changed(&old, &new));

    let mut same = HashMap::new();
    same.insert("a".into(), AttrValue::String("1".into()));
    assert!(!props_changed(&old, &same));
}

#[test]
fn test_escape_html() {
    assert_eq!(escape_html("&"), "&amp;");
    assert_eq!(escape_html("<tag>"), "&lt;tag&gt;");
    assert_eq!(escape_html("hello"), "hello");
}

// ─── FRAGMENT DIFF TESTS ───

#[test]
fn test_fragment_diff() {
    let old = VDomNode::fragment(vec![VDomNode::text("a")]);
    let new = VDomNode::fragment(vec![VDomNode::text("b")]);
    let result = diff(Some(&old), &new);
    assert_eq!(result.patches.len(), 1);
    match &result.patches[0] {
        Patch::SetText { text, .. } => assert_eq!(text, "b"),
        _ => panic!("expected SetText"),
    }
}

#[test]
fn test_fragment_vs_element_diff() {
    let old = VDomNode::fragment(vec![VDomNode::text("a")]);
    let new = VDomNode::element("div", vec![], vec![VDomNode::text("b")]);
    let result = diff(Some(&old), &new);
    assert!(!result.patches.is_empty());
}

// ─── COMPONENT DIFF TESTS ───

#[test]
fn test_component_name_change() {
    let old = VDomNode::component("OldComp", HashMap::new(), None);
    let new = VDomNode::component("NewComp", HashMap::new(), None);
    let result = diff(Some(&old), &new);
    assert_eq!(result.stats.nodes_replaced, 1);
}

// ─── EVENT DIFF TESTS ───

#[test]
fn test_event_handler_diff() {
    let mut old_events = HashMap::new();
    old_events.insert("click".into(), 1u64);
    let old = VDomNode::element_with_attrs(
        "button",
        HashMap::new(),
        old_events,
        vec![],
        None,
    );

    let mut new_events = HashMap::new();
    new_events.insert("click".into(), 2u64);
    let new = VDomNode::element_with_attrs(
        "button",
        HashMap::new(),
        new_events,
        vec![],
        None,
    );

    let result = diff(Some(&old), &new);
    let has_attach = result.patches.iter().any(|p| matches!(p, Patch::AttachEvent { .. }));
    let has_detach = result.patches.iter().any(|p| matches!(p, Patch::DetachEvent { .. }));
    assert!(has_attach, "Should attach new event handler");
    assert!(has_detach, "Should detach old event handler");
}

// ─── SSR PRETTY PRINT ───

#[test]
fn test_ssr_pretty_print() {
    let node = VDomNode::element("div", vec![], vec![
        VDomNode::element("h1", vec![], vec![VDomNode::text("Title")]),
        VDomNode::element("p", vec![], vec![VDomNode::text("Body")]),
    ]);
    let mut renderer = SsrRenderer::new(true);
    let html = renderer.render(&node);

    // Check there's actual indentation
    assert!(html.contains("\n"), "Pretty should add newlines");
}

// ─── LARGE LIST DIFF (PERFORMANCE) ───

#[test]
fn test_large_list_diff() {
    let mut items: Vec<VDomNode> = (0..200)
        .map(|i| {
            VDomNode::Element(ElementNode {
                tag: "li".into(),
                attrs: [("data-id".into(), AttrValue::Number(i as f64))]
                    .iter()
                    .cloned()
                    .collect(),
                events: Default::default(),
                children: vec![VDomNode::text(&format!("Item {}", i))],
                key: Some(format!("key-{}", i)),
            })
        })
        .collect();

    let old = VDomNode::element("ul", vec![], items.clone());

    // Change one item
    items[150] = VDomNode::Element(ElementNode {
        tag: "li".into(),
        attrs: [("data-id".into(), AttrValue::Number(150.0))]
            .iter()
            .cloned()
            .collect(),
        events: Default::default(),
        children: vec![VDomNode::text("Modified Item 150")],
        key: Some("key-150".into()),
    });

    let new = VDomNode::element("ul", vec![], items);

    let start = std::time::Instant::now();
    let result = diff(Some(&old), &new);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_micros() < 50000,
        "Diff 200 items should be fast: took {}μs",
        elapsed.as_micros()
    );
    assert!(
        result.patches.len() < 10,
        "Minimal patches for single item change: got {}",
        result.patches.len()
    );
}
