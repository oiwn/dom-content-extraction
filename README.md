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

## Unicode Support

DOM Content Extraction includes robust Unicode support for handling multilingual content:

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


## Binary Usage

The crate includes a command-line binary tool `dce` (DOM Content Extraction) for extracting main content from HTML documents. It supports both local files and remote URLs as input sources.

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
  -h, --help            Print help
  -V, --version         Print version
```

Note: Either `--url` or `--file` must be specified, but not both.

### Features

- **URL Fetching**: Automatically downloads HTML content from specified URLs
- **Timeout Control**: 30-second timeout for URL fetching to prevent hangs
- **Error Handling**: Comprehensive error messages for common failure cases
- **Flexible Output**: Write to file or stdout
- **Temporary File Management**: Automatic cleanup of downloaded content

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

### Error Handling

The binary provides clear error messages for common scenarios:

- Invalid URLs
- Network timeouts
- File access issues
- HTML parsing errors
- Content extraction failures

### Dependencies

The binary functionality requires the following additional dependencies:

- `clap`: Command-line argument parsing
- `reqwest`: HTTP client for URL fetching
- `tempfile`: Temporary file management
- `url`: URL parsing and validation
- `anyhow`: Error handling

These dependencies are only included when building with the default `cli` feature.

