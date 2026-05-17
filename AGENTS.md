# TriVM — Agent Guide

## Project structure
Single Rust crate (lib only). No workspace. No external dependencies.

- `src/trit.rs` — Trit enum (NegOne/Zero/One) + truth tables
- `src/tryte.rs` — Tryte (i16 newtype, -364..364) + arithmetic
- `src/instruction.rs` — Instruction enum (23 variants) + encode/decode
- `src/vm.rs` — VM state + exec_inst + step + run
- `src/main.rs` — CLI binary (reads .tri file, runs VM, outputs result)

All tests live in-source (`#[cfg(test)] mod tests` inside each `.rs`). No `tests/` directory.

## Commands
- `cargo test` — runs all tests (only verification step needed)
- `cargo build` — builds lib + binary
- `cargo run -- <file.tribin>` — runs VM on a binary program file

## Testing patterns
- Test helpers: `r(i: u8) -> Reg` creates a register, `t(v: i16) -> Tryte` unwraps from_i16
- `place_inst(vm, addr, &inst)` writes encoded instruction to VM memory at given address
- `vm.exec_inst(&inst)` tests single instruction execution
- `vm.step()` tests fetch-decode-execute cycle (use with place_inst)
- `vm.run()` tests full program execution until HALT or error

## Key gotchas
- **Trit is exactly 3 variants** — match must be exhaustive; `_ =>` on Trit is unreachable
- **Tryte range**: -364..=364, not full i16. `from_i16()` returns `Option`.
- **Instruction encoding**: 2 fixed-width trytes (12 trits). First trit selects format (R=0/I=1/C=T).
- **SYSCALL convention**: `Rs` = function code (1=EXIT, 2=PRINT_T, 3=PRINT_D, 4=PRINT_C, 7=PRINT_S), `R0` = argument
- **Memory layout**: 59049 trytes (3^10). .text at 256, .data at 20000, heap at 40000, stack at 55000 down.
- **Docs in `docs/`**: DESIGN.md (architecture), ROADMAP.md (progress), TEST_CASES.md, SYSTEM_ACCEPTANCE.md.

## Layers
trit → tryte → instruction → vm → main (depends on prior; no re-exports from lib.rs)
