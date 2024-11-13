# dom-content-extraction

<div align="center">
   <a href="https://crates.io/crates/dom-content-extraction">
        <img src="https://img.shields.io/crates/dr/dom-content-extraction" alt="Crates.io">
    </a>
  <a href="https://codecov.io/github/oiwn/dom-content-extraction" > 
   <img src="https://codecov.io/github/oiwn/dom-content-extraction/graph/badge.svg?token=6Y7IYX29OP"/> 
   </a>
</div>


A Rust library for extracting main content from web pages using text 
density analysis. This is an implementation of the Content Extraction 
via Text Density (CETD) algorithm described in the paper by Fei Sun, 
Dandan Song and Lejian Liao: 

[Content Extraction via Text Density](http://ofey.me/papers/cetd-sigir11.pdf).

## What Problem Does This Solve?

Web pages often contain a lot of peripheral content like navigation menus, advertisements, footers, and sidebars. This makes it challenging to extract just the main content programmatically. This library helps solve this problem by:

- Analyzing the text density patterns in HTML documents
- Identifying content-rich sections versus navigational/peripheral elements
- Extracting the main content while filtering out noise
- Handling various HTML layouts and structures

## Key Features

- Build a density tree representing text distribution in the HTML document
- Calculate composite text density using multiple metrics
- Extract main content blocks based on density patterns
- Support for nested HTML structures
- Efficient processing of large documents
- Error handling for malformed HTML

## Usage

Basic usage example:

```rust
use dom_content_extraction::{DensityTree, get_node_text};

let dtree = DensityTree::from_document(&document)?; // Takes a scraper::Html document

// Get nodes sorted by text density
let sorted_nodes = dtree.sorted_nodes();
let densest_node = sorted_nodes.last().unwrap();

// Extract text from the node with highest density
println!("{}", get_node_text(densest_node.node_id, &document)?);

// For more accurate content extraction:
dtree.calculate_density_sum()?;
let main_content = dtree.extract_content(&document)?;
println!("{}", main_content);
```

## Installation 

Add it it with:

```bash
cargo add dom-content-extraction
```

or add to you  `Cargo.toml`

```
dom-content-extraction = "0.3"
```

## Documentation

Read the docs! 

[dom-content-extraction documentation](https://docs.rs/dom-content-extraction/latest/dom_content_extraction/)

## Run examples

Check examples.

This one will extract content from generated "lorem ipsum" page

```bash
cargo run --example check -- lorem-ipsum 
```

There is scoring example i'm trying to implement scoring.
You will need to download GoldenStandard and finalrun-input datasets from:

[https://sigwac.org.uk/cleaneval/](https://sigwac.org.uk/cleaneval/)

and unpack archives into `data/` directory.

```bash
cargo run --example ce_score
```

As far as i see there is problem opening some files:

```bash
Error processing file 730: Failed to read file: "data/finalrun-input/730.html"

Caused by:
    stream did not contain valid UTF-8
```

But overall extraction works pretty well:

```text
Overall Performance:
  Files processed: 370
  Average Precision: 0.87
  Average Recall: 0.82
  Average F1 Score: 0.75  
```

[Read documentation on docs.rs](https://docs.rs/dom-content-extraction/latest/dom_content_extraction/)


### Desired features

- [ ] implement normal scoring
- [ ] create real world dataset
