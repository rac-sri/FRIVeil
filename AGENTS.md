# PROJECT KNOWLEDGE BASE

**Generated:** 2026-01-29

## OVERVIEW
Rust 2024 crate implementing a FRI-based data availability sampling PoC (Binius stack).

## STRUCTURE
```
./
├── src/               # Core Rust library + binary
├── benches/           # Divan benchmarks
├── OPTIMIZATION_*.md  # Performance analysis & quick reference
└── benchmark_optimizations.sh  # Perf script (multi-profile)
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Core protocol logic | `src/friveil.rs` | Largest file; FriVeil struct + commit/prove/verify flow |
| MLE / packing utilities | `src/poly.rs` | `Utils` + PackedMLE helpers |
| Public API surface | `src/lib.rs` | `pub mod friveil; pub mod poly;` |
| Binary example flow | `src/main.rs` | End-to-end pipeline with tracing |
| Benchmarks | `benches/commitment.rs` | Divan harness; 32MiB workload |
| Perf guidance | `OPTIMIZATION_ANALYSIS.md` | Hot paths, release vs debug |
| Quick perf commands | `OPTIMIZATION_QUICK_REFERENCE.md` | Run profiles + tips |

## CONVENTIONS (PROJECT-SPECIFIC)
- Rust edition **2024** (see Cargo.toml).
- Optional feature: `parallel` (Rayon). Default features empty.
- Bench harness: **divan**, with `[[bench]] harness = false`.
- Performance work assumes **release** builds and profiling via `release-with-debug`.

## ANTI-PATTERNS (THIS PROJECT)
- Do **not** benchmark in debug mode when comparing performance.
- Do **not** assume `parallel` is always enabled; keep feature gates.
- Do **not** edit external path deps (`../binius/...`) from this repo.

## COMMANDS
```bash
# Run example (debug)
cargo run

# Fast runtime (release)
cargo run --release

# Dev-optimized profile
cargo run --profile=dev-optimized

# Release with debug symbols (profiling)
cargo run --profile=release-with-debug

# Release + parallel feature
cargo run --release --features parallel

# Benchmarks (Divan)
cargo bench

# Multi-profile perf comparison
./benchmark_optimizations.sh
```

## NOTES
- Path dependencies point outside this repo (`../binius/...`); treat as external.
- No CI workflows detected; run commands locally for verification.
