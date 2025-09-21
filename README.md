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
dom-content-extraction = "0.3"
```

### Optional Features

To enable markdown output support:

```toml
dom-content-extraction = { version = "0.3", features = ["markdown"] }
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
  Files processed: 370
  Average Precision: 0.87
  Average Recall: 0.82
  Average F1 Score: 0.75  
```

[Read documentation on docs.rs](https://docs.rs/dom-content-extraction/latest/dom_content_extraction/)


## Binary Usage

The crate includes a command-line binary tool `dce` (DOM Content Extraction) for
extracting main content from HTML documents. It supports both local files and
remote URLs as input sources.

### Installation

The binary is included by default. You can install it using cargo:

```bash
cargo install dom-content-extraction
```

### Command-Line Options

```
dce [OPTIONS]

Options:
  -u, --url <URL>        URL to fetch HTML content from
  -f, --file <FILE>      Local HTML file to process
  -o, --output <FILE>    Output file (stdout if not specified)
      --format <FORMAT>  Output format [default: text] [possible values: text, markdown]
  -h, --help            Print help
  -V, --version         Print version
```

Note: Either `--url` or `--file` must be specified, but not both.

### Markdown Output

To extract content as markdown format, use the `--format markdown` option:

```bash
# Extract as markdown from URL
cargo run --bin dce -- --url "https://example.com" --format markdown

# Extract as markdown from file and save to output
cargo run --bin dce -- --file input.html --format markdown --output content.md
```

Note: Markdown output requires the `markdown` feature to be enabled.

### Features

- **URL Fetching**: Automatically downloads HTML content from specified URLs
- **Timeout Control**: 30-second timeout for URL fetching to prevent hangs
- **Error Handling**: Comprehensive error messages for common failure cases
- **Flexible Output**: Write to file or stdout
- **Temporary File Management**: Automatic cleanup of downloaded content
- **Markdown Support**: Extract content as structured markdown (requires `markdown` feature)

### Examples

Extract content from a URL and print to stdout:
```bash
dce --url "https://example.com/article"
```

Process a local HTML file and save to output file:
```bash
dce --file input.html --output extracted.txt
```

Extract from URL and save directly to file:
```bash
dce --url "https://example.com/page" --output content.txt
```

### Dependencies

The binary functionality requires the following additional dependencies:

- `clap`: Command-line argument parsing
- `reqwest`: HTTP client for URL fetching
- `tempfile`: Temporary file management
- `url`: URL parsing and validation
- `anyhow`: Error handling
- `htmd`: HTML to markdown conversion (for markdown feature)

These dependencies are only included when building with the default `cli`
feature. The `markdown` feature requires the `htmd` dependency.

