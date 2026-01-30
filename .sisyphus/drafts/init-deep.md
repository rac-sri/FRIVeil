# Draft: init-deep AGENTS.md generation

## Requirements (confirmed)
- Run `/init-deep` in update mode (modify existing + create new where warranted).
- MAXIMIZE SEARCH EFFORT with parallel explore/librarian agents and direct Grep/AST-grep/LSP where possible.
- Generate hierarchical AGENTS.md files (root + subdirs as warranted by scoring).
- Use the provided scoring matrix and decision rules for AGENTS.md placement.

## Technical Decisions
- Primary language: Rust (Cargo.toml present; src/*.rs, benches/*.rs).
- Project appears single-crate with lib.rs + main.rs and benches/ (divan harness).
- No CI workflows or Makefile detected.
- No explicit test config discovered; tests are inline in src/friveil.rs and benches use divan.
- Exclude build artifacts and generated outputs from scoring/placement.
- AGENTS.md locations selected: root (`./AGENTS.md`) and `src/AGENTS.md` only.

## Research Findings
- Cargo.toml: edition = "2024"; features include optional "parallel"; bench "commitment" with harness=false; custom profiles.
- Structure: src/main.rs, src/lib.rs, src/friveil.rs, src/poly.rs; benches/commitment.rs.
- Large file hotspot: src/friveil.rs (~857 lines) is the primary complex module.
- Optimization docs: OPTIMIZATION_ANALYSIS.md + OPTIMIZATION_QUICK_REFERENCE.md describe performance hotspots and benchmark commands.
- Bench harness: divan in benches/commitment.rs; benchmark_optimizations.sh script referenced in docs.
- Monorepo/workspace: root crate has path deps to ../binius/... but no [workspace] in this repo.
- No AGENTS.md or CLAUDE.md detected in repo currently.

## Open Questions
- (Resolved) Exclude build artifacts and generated outputs (e.g., target/, node_modules/, dist/) from scoring/placement.

## Scope Boundaries
- INCLUDE: root, src/, benches/, optimization docs, Cargo.toml, benchmark script.
- EXCLUDE: external sibling crates referenced via path deps (../binius/...), node_modules/ and target/ if excluded.
