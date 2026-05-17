# TriVM — Agent Guide

## Project structure
Single Rust crate (lib only). No workspace. No external dependencies.

```
src/
├── lib.rs            # pub mod declarations (no re-exports)
├── trit.rs           # Trit enum (NegOne/Zero/One) + truth tables
├── tryte.rs          # Tryte (i16 newtype, -364..364) + arithmetic
├── instruction.rs    # Instruction enum (23 variants) + encode/decode
├── vm.rs             # VM state + exec_inst + step + run
├── assembler.rs      # Assembler (lexer → parser → codegen)
└── main.rs           # CLI binary (.tri source or .tribin raw)
docs/
├── DESIGN.md         # Full architecture reference
├── ROADMAP.md        # Progress tracker
├── SYSTEM_ACCEPTANCE.md
└── TEST_CASES.md
examples/
├── hello.tri         # PRINT_S + EXIT
├── fib.tri           # Fibonacci iterative
├── ternary.tri       # Balanced ternary arithmetic
├── subroutine.tri    # CALL/RET square
├── dot_product.tri   # MAC dot product
└── demo.tri          # All-instruction showcase
```

## Commands
- `cargo test` — runs all 184 in-source tests (REQUIRED after any change)
- `cargo build` — builds lib + binary (zero warnings required)
- `cargo run -- <file>` — auto-detects `.tri` (assemble+run) vs `.tribin` (load+run)

## Layers (dependency order)
trit → tryte → instruction → vm → assembler → main
Each `src/*.rs` depends only on prior layers. `lib.rs` only does `pub mod`.

## Assembler pipeline
```
source &str → Lexer → Vec<Token> → Parser → Vec<ParsedLine> → Codegen (2-pass) → Vec<Tryte>
```
1. **Lexer**: tokenizes mnemonics, Reg (R0-R7), numbers (decimal/0t/0x/0b/bare T=-1),
   directives (.text/.data/.word/.asciiz), string literals, labels, comments.
2. **Parser**: section tracking, pending_label binding to next instruction/directive,
   operand count validation (HALT/RET reject operands, others require correct count).
3. **Codegen**: pass 1 builds symbol table with addresses; pass 2 emits Tryte pairs and
   resolves label references to relative offsets (offset = target - current_pc).

## Instruction encoding (12 trits = 2 trytes)
First trit selects format:
- **0** = Format R: `| 0 | op 2t | rd 3t | rs 3t | rt 3t |`
- **1** = Format I: `| 1 | op 2t | rd 3t | (rs 3t + imm 3t) or (imm 6t) |`
- **T** = Format C: `| T | op 2t | operand 9t |`

### Format R (op)
| op | inst | semantics |
|----|------|-----------|
| 00 | NOP  | — |
| 01 | ADD  | Rd = Rs + Rt |
| 0T | SUB  | Rd = Rs - Rt |
| 10 | MUL  | Rd = Rs × Rt |
| 1T | DIV  | Rd = Rs / Rt |
| 11 | MOD  | Rd = Rs % Rt |
| T0 | MAC  | Rd += Rs × Rt |
| TT | CMP  | Rd = sgn(Rs - Rt) |
| T1 | SGN  | Rd = sgn(Rs) |

### Format I (op)
| op | width | inst | semantics |
|----|-------|------|-----------|
| 00 | 6t    | LDI Rd, imm  | Rd = imm (-364..364) |
| 0T | 3t    | ADDI Rd,Rs,imm | Rd = Rs + imm (-13..13) |
| 01 | 3t    | MULI Rd,Rs,imm | Rd = Rs × imm |
| 10 | 3t    | LD Rd,[Rs+imm] | Rd = mem[Rs+imm] |
| 1T | 3t    | ST Rs,[Rd+imm] | mem[Rd+imm] = Rs |

### Format C (op)
| op | inst | semantics |
|----|------|-----------|
| 00 | JMP offset | PC += offset (±364) |
| 0T | JMP Rs | PC = Rs (indirect) |
| 01 | BZ Rs,offset | if Rs==0: PC += offset |
| 10 | BN Rs,offset | if Rs<0: PC += offset |
| 1T | BP Rs,offset | if Rs>0: PC += offset |
| 11 | CALL offset | push PC+2; PC += offset |
| T0 | RET | pop PC |
| TT | SYSCALL Rs | see syscall table |
| T1 | HALT | stop execution |

### Syscall (Rs = function code)
| code | name | description |
|------|------|-------------|
| 1 | EXIT | terminate, R0 = exit code |
| 2 | PRINT_T | print R0 in balanced ternary |
| 3 | PRINT_D | print R0 as decimal integer |
| 4 | PRINT_C | print R0 as ASCII char |
| 7 | PRINT_S | print null-terminated string at mem[R0] |

## Key gotchas
- **Trit is exactly 3 variants** — match must be exhaustive; `_ =>` on Trit is unreachable.
- **Tryte range**: -364..=364, not full i16. `from_i16()` returns `Option`.
- **Assembler output is contiguous** from address 0 (not split at 256/20000). Programs
  >~182 instructions may exceed Tryte range for label addresses (LDI limitation).
- **Branch offset** = target - current_pc (PC not incremented before adding offset).
- **SYSCALL Rs**: function code goes in Rs register, argument in R0.
- **Bare T** in source = -1 (balanced ternary digit). Pure digit strings like `10` are
  always decimal 10, never parsed as balanced ternary 3.

## Common tasks

### Add a new instruction
1. Add variant to `Instruction` enum in `instruction.rs`
2. Add encode/decode branches
3. Add exec branch in `vm.rs` `exec_inst`
4. Add assembler mnemonic in `assembler.rs` `resolve_instruction` + operand template
5. Write tests at each layer (instruction round-trip, VM execution, assembler codegen)

### Write a new assembly example
1. Create `examples/foo.tri`
2. Verify: `cargo run -- examples/foo.tri`
3. Ensure all label addresses stay within Tryte range (check with `cargo test`)
4. Add to `docs/SYSTEM_ACCEPTANCE.md` if it covers a new scenario

### Fix a codegen bug
1. Add regression test first (failing)
2. Fix the lexer/parser/codegen logic
3. Run `cargo test` — all 184 tests must pass, zero new warnings

## Testing patterns
- Helpers: `r(i: u8) -> Reg`, `t(v: i16) -> Tryte` via from_i16 unwrap
- VM tests: `place_inst(vm, addr, &inst)`, then `vm.exec_inst(&inst)` or `vm.step()`
- Assembler tests: `assemble("...")` returns `Result<Vec<Tryte>, AssembleError>`
- Assertions: `assert_eq!(vm.regs[0], t(42))`, `assert!(result.is_err())`
