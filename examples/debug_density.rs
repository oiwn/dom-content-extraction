use dom_content_extraction::{DensityTree, get_node_text};
use scraper::Html;
use std::fs;

fn main() {
    let html_content =
        fs::read_to_string("html/test_1.html").expect("Unable to read file");
    let document = Html::parse_document(&html_content);
    let mut dtree = DensityTree::from_document(&document).unwrap();
    dtree.calculate_density_sum().unwrap();

    println!("Density analysis for test_1.html:");
    println!("================================");

    // Get nodes sorted by density sum
    let sorted_nodes = dtree.sorted_nodes();

    for (i, node) in sorted_nodes.iter().enumerate() {
        if let Ok(text) = get_node_text(node.node_id, &document) {
            if !text.trim().is_empty() {
                println!(
                    "\nNode {} (density_sum: {:.2}):",
                    i,
                    node.density_sum.unwrap_or(0.0)
                );
                println!("Text: '{}'", text.trim());
            }
        }
    }

    // Show the max density node
    if let Some(max_node) = dtree.get_max_density_sum_node() {
        println!("\n=== MAX DENSITY NODE ===");
        println!(
            "Density sum: {:.2}",
            max_node.value().density_sum.unwrap_or(0.0)
        );
        if let Ok(text) = get_node_text(max_node.value().node_id, &document) {
            println!("Content: '{}'", text.trim());
        }
    }
}
