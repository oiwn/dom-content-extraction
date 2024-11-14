use clap::{Parser, Subcommand};
use dom_content_extraction::{scraper::Html, DensityTree};
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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::LoremIpsum => {
            println!("Processing Lorem Ipsum example...");
            process_lorem_ipsum();
        }
    }
}

fn process_lorem_ipsum() {
    let html_content =
        fs::read_to_string("html/lorem_ipsum.html").expect("Unable to read file");
    let document = Html::parse_document(&html_content);
    let mut dtree = DensityTree::from_document(&document).unwrap();
    let _ = dtree.calculate_density_sum();
    let extracted_content = dtree.extract_content(&document).unwrap();
    println!("Extracted content:\n{}", extracted_content);
}
