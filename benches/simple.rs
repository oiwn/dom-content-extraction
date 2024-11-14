use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{fs, io::Read, path};
use zip::read::ZipArchive;

use dom_content_extraction::*;

pub fn read_file(
    file_path: impl AsRef<path::Path>,
) -> Result<String, std::io::Error> {
    let content: String = fs::read_to_string(file_path)?;
    Ok(content)
}

pub fn build_dom(html: &str) -> scraper::Html {
    let document: scraper::Html = scraper::Html::parse_document(html);
    document
}

fn read_file_content_from_zip(zip_path: &str, file_name: &str) -> Option<String> {
    let zipfile = fs::File::open(zip_path).unwrap();
    let mut archive = ZipArchive::new(zipfile).unwrap();

    let result = match archive.by_name(file_name) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            Some(content.to_string())
        }
        Err(..) => None,
    };
    result
}

fn benchmark_test_1_html_dom_content_extaction(c: &mut Criterion) {
    let content = read_file("html/test_1.html").unwrap();
    c.bench_function("test_1_dom_content_extraction", |b| {
        b.iter(|| {
            let document = build_dom(black_box(content.as_str()));

            let dtree = DensityTree::from_document(&document).unwrap();
            let sorted_nodes = dtree.sorted_nodes();
            let node_id = sorted_nodes.last().unwrap().node_id;
            assert_eq!(get_node_text(node_id, &document).unwrap().len(), 200);
        })
    });
}

fn benchmark_real_file_dom_content_extraction(c: &mut Criterion) {
    let content = read_file_content_from_zip(
        "html/pages.zip",
        "pages/china-beating-us-nigerian-lithium-rush-race-go-electric.html",
    )
    .unwrap();

    c.bench_function("real_file_dom_content_extraction", |b| {
        b.iter(|| {
            let document = build_dom(black_box(content.as_str()));

            let dtree = DensityTree::from_document(&document).unwrap();
            let sorted_nodes = dtree.sorted_nodes();
            let node_id = sorted_nodes.last().unwrap().node_id;
            assert!(!get_node_text(node_id, &document).unwrap().is_empty());
        })
    });
}

fn benchmark_real_file_density_tree_calculation(c: &mut Criterion) {
    let content = read_file_content_from_zip(
        "html/pages.zip",
        "pages/sas-bankruptcy-protection.html",
    )
    .unwrap();
    let document = build_dom(content.as_str());

    c.bench_function("real_file_density_tree_calculation", |b| {
        b.iter(|| {
            let dtree = DensityTree::from_document(black_box(&document)).unwrap();
            assert_eq!(dtree.tree.values().len(), 893);
        })
    });
}

fn benchmark_real_file_density_tree_calculation_and_sort(c: &mut Criterion) {
    let content = read_file_content_from_zip(
        "html/pages.zip",
        "pages/sas-bankruptcy-protection.html",
    )
    .unwrap();
    let document = build_dom(content.as_str());

    c.bench_function("real_file_density_tree_sort_nodes", |b| {
        b.iter(|| {
            let dtree = DensityTree::from_document(black_box(&document)).unwrap();
            let sorted_nodes = dtree.sorted_nodes();
            let last_node = sorted_nodes.last().unwrap();
            assert_eq!(last_node.density, 104.79147);
        })
    });
}

fn benchmark_node_text_extraction(c: &mut Criterion) {
    let content = read_file_content_from_zip(
        "html/pages.zip",
        "pages/sas-bankruptcy-protection.html",
    )
    .unwrap();
    let document = build_dom(content.as_str());

    let dtree = DensityTree::from_document(&document).unwrap();
    let sorted_nodes = dtree.sorted_nodes();
    let last_node_id = sorted_nodes.last().unwrap().node_id;

    c.bench_function("real_file_density_tree_sort_and_text_extraction", |b| {
        b.iter(|| {
            let node_text =
                get_node_text(black_box(last_node_id), black_box(&document))
                    .unwrap();
            assert_eq!(node_text.len(), 3065);
        })
    });
}

criterion_group!(
    benches,
    benchmark_test_1_html_dom_content_extaction,
    benchmark_real_file_dom_content_extraction,
    benchmark_real_file_density_tree_calculation,
    benchmark_real_file_density_tree_calculation_and_sort,
    benchmark_node_text_extraction,
);

criterion_main!(benches);
