# Current task context

## Current direction

Focused library crate: DOM/text-density extraction and optional markdown output.
CLI responsibilities moved to [`pginf`](https://github.com/oiwn/pageinfo-rs).

## Verification status

All gates pass:

```bash
cargo test --all-features    # 39 tests
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Pre-commit hooks via `prek`: `fmt`, `clippy`, `typos`, `gitleaks`.

## Real-page examples checked

```text
https://caseelegance.com/blogs/humidor-resources/building-a-humidor-diy
https://brianholcombewoodworker.com/2016/03/31/humidor-build-casework/
https://www.lumberjocks.com/threads/making-a-humidor.362341/
```

- Case Elegance: clean article extraction, no JS/CSS/SVG pollution. Boundary is broad (includes related/product merchandising).
- Brian Holcombe: article text and comments extracted cleanly. Whether comments should remain is a future policy decision.
- LumberJocks: clean extraction from saved HTML. URL fetching (`406 Not Acceptable`) is a client concern for `pginf`.

## Next work in this crate

1. Increase unit test density for core algorithms and edge cases.
2. Profile memory usage and benchmark performance across document sizes.
3. Caller-controlled extraction policy (see `specs/ideas.md`).
4. Python bindings via PyO3 (see `specs/ideas.md`).

## Work for `pginf`

Implement CLI extraction using this crate as a dependency:

```toml
dom-content-extraction = { version = "...", default-features = false, features = ["markdown"] }
```

`pginf` owns: HTTP fetching, browser/TLS emulation, encoding detection, retries, CLI UX, diagnostics.
This crate owns: HTML parsing, density analysis, content extraction heuristics.
