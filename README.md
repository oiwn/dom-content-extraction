# dom-content-extraction

Rust implementation of Fei Sun, Dandan Song and Lejian Liao paper:

[Content Extraction via Text Density (CETD)](http://ofey.me/papers/cetd-sigir11.pdf)

```rust
use dom_content_extraction::{DensityTree, get_node_text};

let dtree = DensityTree::from_document(&document); // &scraper::Html 
let sorted_nodes = dtree.sorted_nodes();
let node_id = sorted_nodes.last().unwrap().node_id;

println!("{}", get_node_text(node_id, &document));
```

[Read documentation on docs.rs](https://docs.rs/dom-content-extraction/latest/dom_content_extraction/)
