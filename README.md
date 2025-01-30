# dom-content-extraction

![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/dom-content-extraction)
![GitHub License](https://img.shields.io/github/license/oiwn/dom-content-extraction)
[![codecov](https://codecov.io/gh/oiwn/dom-content-extraction/graph/badge.svg?token=6Y7IYX29OP)](https://codecov.io/gh/oiwn/dom-content-extraction)
[![dependency status](https://deps.rs/repo/github/oiwn/dom-content-extraction/status.svg)](https://deps.rs/repo/github/oiwn/dom-content-extraction)

A Rust library for extracting main content from web pages using text 
density analysis. This is an implementation of the Content Extraction 
via Text Density (CETD) algorithm described in the paper by 
[Fei Sun, Dandan Song and Lejian Liao: Content Extraction via Text Density](https://ofey.me/assets/pdf/cetd-sigir11.pdf).

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

Due to "LazyLock" MSRV is 1.80

Basic usage example:

```rust
use scraper::Html;
use dom_content_extraction::{DensityTree, get_node_text};

let document = Html::parse_document(&html_content);
let mut dtree = DensityTree::from_document(&document).unwrap();
let _ = dtree.calculate_density_sum();
let extracted_content = dtree.extract_content(&document).unwrap();
println!("Extracted content:\n{}", extracted_content);
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

This one print node with highest density:

```bash
cargo run --examples check -- test4
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
