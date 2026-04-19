# dom-content-extraction

<div align="center">
   <a href="https://crates.io/crates/dom-content-extraction">
        <img src="https://img.shields.io/crates/dr/dom-content-extraction" alt="Crates.io">
    </a>
  <a href="https://codecov.io/github/oiwn/dom-content-extraction" > 
   <img src="https://codecov.io/github/oiwn/dom-content-extraction/graph/badge.svg?token=6Y7IYX29OP"/> 
   </a>
  [![dependency status](https://deps.rs/repo/github/oiwn/dom-content-extraction/status.svg)](https://deps.rs/repo/github/oiwn/dom-content-extraction)
</div>


A Rust library for extracting main content from web pages using text 
density analysis. This is an implementation of the Content Extraction 
via Text Density (CETD) algorithm described in the paper by 
[Fei Sun, Dandan Song and Lejian Liao: Content Extraction via Text Density](http://ofey.me/papers/cetd-sigir11.pdf).

See [`specs/overview.md`](specs/overview.md) for detailed architecture and internals.

## What Problem Does This Solve?

Web pages often contain a lot of peripheral content like navigation menus,
advertisements, footers, and sidebars. This makes it challenging to extract just
the main content programmatically. This library helps solve this problem by:

- Analyzing the text density patterns in HTML documents
- Identifying content-rich sections versus navigational/peripheral elements
- Extracting the main content while filtering out noise
- Handling various HTML layouts and structures

## Key Features

- Build a density tree representing text distribution in the HTML document
- Calculate composite text density using multiple metrics
- Extract main content blocks based on density patterns
- Unicode Support
- Support for nested HTML structures
- Efficient processing of large documents
- Error handling for malformed HTML
- **Markdown output** (optional feature) - Extract content as structured markdown

## Unicode Support

DOM Content Extraction includes Unicode support for handling multilingual content:

- Proper character counting using Unicode grapheme clusters
- Unicode normalization (NFC) for consistent text representation
- Support for various writing systems including Latin, Cyrillic, and CJK scripts
- Accurate text density calculations across different languages

This ensures accurate content extraction from web pages in any language, with proper handling of:

- Combining characters (like accents in European languages)
- Bidirectional text
- Complex script rendering
- Multi-code-point graphemes (like emojis)

## Usage

MSRV is 1.85 due to 2024 edition. Living on the edge!

Basic usage example:

```rust
use scraper::Html;
use dom_content_extraction::get_content;

fn main() {
    let html = r#"<!DOCTYPE html><html><body>
        <nav>Home | About</nav>
        <main>
            <article>
                <h1>Main Article</h1>
                <p>This is the primary content that contains enough text to maintain proper density metrics. The paragraph needs sufficient length to establish text-to-link ratio.</p>
                <p>Second paragraph adds more textual density to ensure the content extraction algorithm works correctly.</p>
                <a href="\#">Related link</a>
            </article>
        </main>
        <footer>Copyright 2024</footer>
    </body></html>"#;

    let document = Html::parse_document(html);
    let content = get_content(&document).unwrap();
    println!("{}", content);
}
```

## Installation 

Add it it with:

```bash
cargo add dom-content-extraction
```

or add to you  `Cargo.toml`

```toml
dom-content-extraction = "0.4"
```

### Optional Features

To enable markdown output support:

```toml
dom-content-extraction = { version = "0.4", features = ["markdown"] }
```

## Documentation

Read the docs! 

[dom-content-extraction documentation](https://docs.rs/dom-content-extraction/latest/dom_content_extraction/)

### Library Usage with Markdown

```rust
use dom_content_extraction::{DensityTree, extract_content_as_markdown, scraper::Html};

let html = "<html><body><article><h1>Title</h1><p>Content</p></article></body></html>";
let document = Html::parse_document(html);
let mut dtree = DensityTree::from_document(&document)?;
dtree.calculate_density_sum()?;

// Extract as markdown
let markdown = extract_content_as_markdown(&dtree, &document)?;
println!("{}", markdown);
# Ok::<(), dom_content_extraction::DomExtractionError>(())
```

## Run examples

Check examples.

This one will extract content from generated "lorem ipsum" page

```bash
cargo run --example check -- lorem-ipsum 
```

This one prints node with highest density:

```bash
cargo run --example check -- test4
```

Extract content as markdown from lorem ipsum (requires markdown feature):

```bash
cargo run --example check -- lorem-ipsum-markdown
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
  Files processed: 653
  Average Precision: 0.88
  Average Recall: 0.83
  Average F1 Score: 0.78
  Average Sorensen-Dice: 0.79
Total processing time: 11.32s
Average time per file: 17.34ms
```

[Read documentation on docs.rs](https://docs.rs/dom-content-extraction/latest/dom_content_extraction/)

## CLI Tool

For command-line usage (URL fetching, file processing, encoding detection), see [`pageinfo-rs`](https://github.com/oiwn/pageinfo-rs).

