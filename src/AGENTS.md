# SRC MODULE GUIDE

## OVERVIEW
Core implementation of FriVeil protocol and MLE utilities.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| FriVeil protocol core | `friveil.rs` | Commit/prove/verify + transcript flows |
| MLE conversion utils | `poly.rs` | `Utils` + PackedMLE + bytes→MLE pipeline |
| Public exports | `lib.rs` | `pub mod friveil; pub mod poly;` |
| Example execution | `main.rs` | End-to-end run with tracing + benchmarks | 

## CONVENTIONS
- Modules are file-backed (`friveil.rs`, `poly.rs`) and re-exported in `lib.rs`.
- The binary (`main.rs`) declares `mod friveil; mod poly;` for local use.
- SIMD/field abstractions rely on Binius traits: `PackedField`, `ExtensionField`.
- Feature-gated parallelism (`parallel`) is used in hot paths—keep `#[cfg(feature = "parallel")]` guards.

## ANTI-PATTERNS
- Don’t add new modules without updating `lib.rs` exports and `main.rs` module declarations.
- Don’t bypass `PackedField`/`ExtensionField` constraints; SIMD assumptions matter.
- Don’t remove feature gates around Rayon paths; keep serial fallback.
