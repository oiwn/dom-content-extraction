# Random ideas

## Python bindings

Add Python bindings via PyO3 in the same crate, behind an opt-in `python` feature.

Why same repo:

- the API surface is tiny (`get_content`, `extract_markdown`) — not worth a separate crate
- single version, single `Cargo.toml`, single release
- follows the pattern used by `polars`, `tokenizers`, `pydantic-core`

Implementation sketch:

- add `pyo3` as an optional dependency gated behind a `python` feature
- gate a `src/python.rs` module with `#[cfg(feature = "python")]`
- expose `get_content(html: &str) -> String` and optionally `extract_markdown(html: &str) -> String`
- add a `pyproject.toml` alongside `Cargo.toml` for `maturin` builds
- Rust users ignore the feature; Python users `pip install` via maturin building from the same source
- `cargo publish` continues to work — `python` is opt-in, default stays `[]`

## Caller-controlled extraction policy

The current hardening work mixes two decisions:

- structural skips for universally non-content DOM nodes (`script`, `style`, `svg`, hidden/ad/share containers)
- heuristic text filtering for machine-like blobs that appear inside otherwise visible text nodes

Longer term, the caller should be able to decide how aggressive extraction should be. A conservative default can avoid false positives for programming prose and forum/code content, while stricter modes can remove naked JS/config/CSS blobs, comments, related content, product grids, and other boilerplate.

Potential shape:

- default policy: structural skips plus very high-confidence blob filtering
- strict article policy: additionally skip comments, related posts, share widgets, recommendations, product cards, and low-content chrome
- custom policy: caller-provided predicate or options for element skipping and text-fragment filtering

For now, improve the current heuristics without adding public API surface. Use syntax/shape evidence rather than broad keywords:

- high punctuation density
- multiple assignments/calls
- braces, semicolons, and JS call syntax
- long encoded/base64-like tokens
- known ad/plugin identifiers only as supporting evidence

Avoid filtering on words like `function`, `window`, `document`, `script`, `style`, or `const` by themselves.
