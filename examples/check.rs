#[cfg(feature = "markdown")]
use dom_content_extraction::extract_content_as_markdown;
use dom_content_extraction::{
    DensityTree, get_content, get_node_text, scraper::Html,
};
use std::fs;

fn process_lorem_ipsum() {
    println!("Processing Lorem Ipsum example...");
    let html_content =
        fs::read_to_string("html/lorem_ipsum.html").expect("Unable to read file");
    let document = Html::parse_document(&html_content);
    let mut dtree = DensityTree::from_document(&document).unwrap();
    let _ = dtree.calculate_density_sum();
    let extracted_content = dtree.extract_content(&document).unwrap();
    println!("Extracted content:\n{}", extracted_content);
}

fn process_test_4_html() {
    println!("Processing test_4 example...");
    let html_content =
        fs::read_to_string("html/test_4.html").expect("Unable to read file");
    let document = Html::parse_document(&html_content);
    let dtree = DensityTree::from_document(&document).unwrap();

    let sorted_nodes = dtree.sorted_nodes();
    let densest_node = sorted_nodes.last().unwrap();

    println!(
        "Highest density node:\n{}",
        get_node_text(densest_node.node_id, &document).unwrap()
    );
}

fn process_toy() {
    let html = r#"
        <!DOCTYPE html><html><body>
            <nav>Navigation</nav>
            <article>
                <h1>Main Article</h1>
                <p>This is the primary content that should be extracted.</p>
            </article>
            <footer>Footer</footer>
        </body></html>
    "#;

    println!("Extracting content from toy html: \n{}\n ", html);
    let document = Html::parse_document(html);
    let content = get_content(&document).unwrap();
    println!("{}", content);
}

#[cfg(feature = "markdown")]
fn process_lorem_ipsum_markdown() {
    println!("Processing Lorem Ipsum example as Markdown...");
    let html_content =
        fs::read_to_string("html/lorem_ipsum.html").expect("Unable to read file");
    let document = Html::parse_document(&html_content);
    let mut dtree = DensityTree::from_document(&document).unwrap();
    dtree.calculate_density_sum().unwrap();

    let markdown_content = extract_content_as_markdown(&dtree, &document).unwrap();
    println!("Extracted markdown content:\n{}", markdown_content);
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();

    match arg.as_str() {
        "lorem-ipsum" => process_lorem_ipsum(),
        "test4" => process_test_4_html(),
        "toy" => process_toy(),
        #[cfg(feature = "markdown")]
        "lorem-ipsum-markdown" => process_lorem_ipsum_markdown(),
        _ => {
            eprintln!("Usage: check <lorem-ipsum|test4|toy|lorem-ipsum-markdown>");
            std::process::exit(1);
        }
    }
}
