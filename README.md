# dom-content-extraction

<div align="center">
 <a href="https://github.com/oiwn/dom-content-extraction/actions/workflows/ci.yml">
        <img src="https://img.shields.io/github/checks-status/oiwn/dom-content-extraction/main" alt="GitHub branch checks state">
    </a>
    |
    <a href="https://crates.io/crates/dom-content-extraction">
        <img src="https://img.shields.io/crates/dr/dom-content-extraction" alt="Crates.io">
    </a>
</div>

Rust implementation of Fei Sun, Dandan Song and Lejian Liao paper:

[Content Extraction via Text Density (CETD)](http://ofey.me/papers/cetd-sigir11.pdf)

```rust
use dom_content_extraction::{DensityTree, get_node_text};

let dtree = DensityTree::from_document(&document); // &scraper::Html 
let sorted_nodes = dtree.sorted_nodes();
let node_id = sorted_nodes.last().unwrap().node_id;

println!("{}", get_node_text(node_id, &document));

dtree.calculate_density_sum();
let extracted_content = dtree.extract_content(&document);

println!("{}", extracted_content;
```
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
- [ ] improve algo
