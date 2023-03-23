use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scraper::Html;
use std::{fs, path};

use dom_content_extraction::*;

pub fn read_file(
    file_path: impl AsRef<path::Path>,
) -> Result<String, std::io::Error> {
    let content: String = fs::read_to_string(file_path)?;
    Ok(content)
}

pub fn build_dom(html: &str) -> Html {
    let document: Html = Html::parse_document(html);
    document
}

fn your_function_benchmark(c: &mut Criterion) {
    let content = read_file("html/test_1.html").unwrap();
    c.bench_function("find_and_extract_text", |b| {
        b.iter(|| {
            // Call your function with some input, e.g.:
            // your_function(black_box(42));

            let document = build_dom(content.as_str());

            let dtree = DensityTree::from_document(&document);
            let sorted_nodes = dtree.sorted_nodes();
            let node_id = sorted_nodes.last().unwrap().node_id;
            assert_eq!(get_node_text(node_id, &document).len(), 200);
        })
    });
}

criterion_group!(benches, your_function_benchmark);
criterion_main!(benches);
