use std::time::Instant;
use rakit_vdom::{h, text, VDomNode};

pub fn bench_render_1000_elements() -> usize {
    let start = Instant::now();

    let items: Vec<VDomNode> = (0..1000).map(|i| {
        h("li", vec![("key", &format!("item-{}", i))],
            vec![text(&format!("Item {}", i))])
    }).collect();

    let _list = h("ul", vec![], items);
    start.elapsed().as_micros() as usize
}

pub fn bench_render_10000_elements() -> usize {
    let start = Instant::now();

    let items: Vec<VDomNode> = (0..10000).map(|i| {
        h("li", vec![("key", &format!("item-{}", i))],
            vec![text(&format!("Item {}", i))])
    }).collect();

    let _list = h("ul", vec![], items);
    start.elapsed().as_micros() as usize
}

pub fn run_all_render_benches() {
    let us = bench_render_1000_elements();
    println!("render 1000 elements: {} us", us);

    let us = bench_render_10000_elements();
    println!("render 10000 elements: {} us", us);
}
