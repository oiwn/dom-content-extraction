# dom-content-extraction

Rust implementation of Fei Sun, Dandan Song and Lejian Liao paper:

[Content Extraction via Text Density (CETD)](http://ofey.me/papers/cetd-sigir11.pdf)

```rust
use density_tree;

let dtree = density_tree:DensityTree::from_document(&document); // &scraper::Html 
let sorted_nodes = dtree.sorted_nodes();
let node_id = sorted_nodes.last().unwrap();

println!("{}", density_tree::get_node_text(node_id, &document));
```
