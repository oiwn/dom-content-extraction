# Current Task Context: Harden dependency and CI supply-chain controls
State: complete

## Plan
- [x] Update direct dependency requirements and refresh the lockfile to audited releases
- [x] Add a scheduled cargo-audit workflow with lockfile enforcement
- [x] Pin CI actions, minimize workflow permissions, and enforce locked Cargo resolution
- [x] Extend Dependabot to GitHub Actions updates
- [x] Run audit, formatting, checks, and tests on supported toolchains/features
- [x] Use readable release/channel references for GitHub Actions
- [x] Add the OpenCode comment and TODO-to-issues workflow
- [x] Declare and continuously verify MSRV 1.88
- [x] Add and validate an explicit cargo-deny policy
