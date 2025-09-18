use clap::{Parser, Subcommand};
use dom_content_extraction::{
    DensityTree, get_content, get_node_text, scraper::Html,
};
use std::fs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    LoremIpsum,
    Test4,
    TestToy,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::LoremIpsum => {
            process_lorem_ipsum();
        }
        Commands::Test4 => {
            process_test_4_html();
        }
        Commands::TestToy => {
            process_toy();
        }
    }
}

fn process_lorem_ipsum() {
    println!("Processing Lorem Ipsum example...");
    let html_content =
        fs::read_to_string("html/lorem_ipsum.html").expect("Unable to read file");
    let document = Html::parse_document(&html_content);
    let mut dtree = DensityTree::from_document(&document).unwrap();
    let _ = dtree.calculate_density_sum(); // do not forget to calculate DS
    let extracted_content = dtree.extract_content(&document).unwrap();
    println!("Extracted content:\n{}", extracted_content);
}

fn process_test_4_html() {
    println!("Processing test_4 example...");
    let html_content =
        fs::read_to_string("html/test_4.html").expect("Unable to read file");
    let document = Html::parse_document(&html_content);
    let dtree = DensityTree::from_document(&document).unwrap();

    // Get nodes sorted by text density
    let sorted_nodes = dtree.sorted_nodes();
    let densest_node = sorted_nodes.last().unwrap();

    // Extract text from the node with highest density
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
