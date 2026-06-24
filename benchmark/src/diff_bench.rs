use std::time::Instant;
use rakit_vdom::{h, text, VDomNode, diff, DiffResult};

fn create_list(n: usize) -> VDomNode {
    let items: Vec<VDomNode> = (0..n).map(|i| {
        h("li", vec![("key", &format!("item-{}", i))],
            vec![text(&format!("Item {}", i))])
    }).collect();
    h("ul", vec![], items)
}

fn create_list_with_change(n: usize, change_at: usize) -> VDomNode {
    let items: Vec<VDomNode> = (0..n).map(|i| {
        if i == change_at {
            h("li", vec![("key", &format!("item-{}", i))],
                vec![text(&format!("Item {} changed!", i))])
        } else {
            h("li", vec![("key", &format!("item-{}", i))],
                vec![text(&format!("Item {}", i))])
        }
    }).collect();
    h("ul", vec![], items)
}

fn create_list_reversed(n: usize) -> VDomNode {
    let items: Vec<VDomNode> = (0..n).rev().map(|i| {
        h("li", vec![("key", &format!("item-{}", i))],
            vec![text(&format!("Item {}", i))])
    }).collect();
    h("ul", vec![], items)
}

pub fn bench_diff_1000_items_single_change() -> usize {
    let old = create_list(1000);
    let new = create_list_with_change(1000, 500);

    let start = Instant::now();
    let _result = diff(Some(&old), &new);
    start.elapsed().as_micros() as usize
}

pub fn bench_diff_1000_items_full_reorder() -> usize {
    let old = create_list(1000);
    let new = create_list_reversed(1000);

    let start = Instant::now();
    let _result = diff(Some(&old), &new);
    start.elapsed().as_micros() as usize
}

pub fn run_all_diff_benches() {
    let us = bench_diff_1000_items_single_change();
    println!("diff 1000 items — 1 change: {} us", us);

    let us = bench_diff_1000_items_full_reorder();
    println!("diff 1000 items — full reorder: {} us", us);
}
