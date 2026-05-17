# TriVM — Balanced Ternary Virtual Machine

![GitHub stars](https://img.shields.io/github/stars/marong103317-ops/tri-vm?style=social)
![GitHub forks](https://img.shields.io/github/forks/marong103317-ops/tri-vm?style=social)
![GitHub license](https://img.shields.io/github/license/marong103317-ops/tri-vm)
![GitHub release](https://img.shields.io/github/v/release/marong103317-ops/tri-vm)
![Rust](https://img.shields.io/badge/language-Rust-orange)

> A minimal **balanced ternary virtual machine** implemented in Rust on binary hardware. Proving ternary computing can work on binary machines, and exploring the possibilities of **ternary AI** and **neural networks**.

## 🚀 Features

- **Ternary Computing**: Native support for balanced ternary arithmetic (-1, 0, 1)
- **Complete VM**: Full instruction set with 23 instructions
- **Assembler**: Built-in assembler for `.tri` source files
- **CLI**: Run ternary programs directly from source or compiled binaries
- **AI Primitives**: MAC (Multiply-Accumulate) operations for neural networks
- **190 Tests**: Comprehensive test coverage

## 🎯 Keywords & Tags

`ternary`, `balanced-ternary`, `virtual-machine`, `VM`, `Rust`, `AI`, `neural-network`, `machine-learning`, `computer-architecture`, `esoteric-language`, `CPU-emulation`, `assembler`, `instruction-set`, `Trit`, `Tryte`, `MAC`, `multiply-accumulate`, `ternary-computing`, `alternative-computing`, `quantum-computing-adjacent`

---

## 🏗️ Core Concepts

### Trit — The Ternary Bit

The fundamental unit of balanced ternary, with three states: **T** (-1), **0** (zero), **1** (one).

| Operation | Rule |
|-----------|------|
| Negation | T↔1, 0→0 |
| Addition | Full truth table with carry |
| Multiplication | T×T=1, T×1=T, others=0 |

**Encoding**: 2-bit binary (00=T, 01=0, 10=1, 11=invalid)

### Tryte — The Ternary Word

6 trits = 1 tryte, range **-364 to 364**. Stored internally as `i16`.

### Registers

| Register | Width | Purpose |
|----------|-------|---------|
| R0-R7 | 6 trits | General purpose (R7 = overflow flag) |
| PC | 12 trits | Program Counter |
| SP | 12 trits | Stack Pointer |

### Memory Layout

```
Address 0     Reserved
256           .text segment
20000         .data segment
40000         Heap
55000         Stack (grows downward)
59048         Max address (3^10 - 1)
```

---

## 📋 Instruction Set

Fixed-width 2 trytes = 12 trits. First trit determines format:

- **0** → Format R: Arithmetic/Logic/Compare (3 register operands)
- **1** → Format I: Immediate/Memory (register + immediate)
- **T** → Format C: Control Flow

### Format R Instructions

| Opcode | Mnemonic | Semantics |
|--------|----------|-----------|
| 00 | `NOP` | No operation |
| 01 | `ADD` | Rd = Rs + Rt |
| 0T | `SUB` | Rd = Rs - Rt |
| 10 | `MUL` | Rd = Rs × Rt |
| 1T | `DIV` | Rd = Rs / Rt |
| 11 | `MOD` | Rd = Rs % Rt |
| T0 | `MAC` | Rd += Rs × Rt (Multiply-Accumulate) |
| TT | `CMP` | Rd = sgn(Rs - Rt) |
| T1 | `SGN` | Rd = sgn(Rs) |

### Format I Instructions

| Opcode | Mnemonic | Semantics |
|--------|----------|-----------|
| 00 | `LDI Rd, imm` | Rd = immediate (-364..364) |
| 0T | `ADDI Rd, Rs, imm` | Rd = Rs + imm (-13..13) |
| 01 | `MULI Rd, Rs, imm` | Rd = Rs × imm |
| 10 | `LD Rd, [Rs+imm]` | Rd = memory[Rs+imm] |
| 1T | `ST Rs, [Rd+imm]` | memory[Rd+imm] = Rs |

### Format C Instructions

| Opcode | Mnemonic | Semantics |
|--------|----------|-----------|
| 00 | `JMP offset` | PC += offset |
| 0T | `JMP Rs` | PC = Rs (indirect) |
| 01 | `BZ Rs, offset` | Branch if Rs == 0 |
| 10 | `BN Rs, offset` | Branch if Rs < 0 |
| 1T | `BP Rs, offset` | Branch if Rs > 0 |
| 11 | `CALL offset` | Push PC+2, PC += offset |
| T0 | `RET` | Pop PC |
| TT | `SYSCALL Rs` | System call |
| T1 | `HALT` | Stop execution |

### System Calls (SYSCALL Rs)

| Code | Name | Description |
|------|------|-------------|
| 1 | `EXIT` | Terminate, R0 = exit code |
| 2 | `PRINT_T` | Print R0 in balanced ternary |
| 3 | `PRINT_D` | Print R0 as decimal |
| 4 | `PRINT_C` | Print R0 as ASCII character |
| 7 | `PRINT_S` | Print null-terminated string |

---

## ⚡ Quick Start

```bash
# Build the VM
cargo build --release

# Run all tests (190 tests)
cargo test

# Run assembly source file
cargo run -- examples/hello.tri

# Run pre-compiled binary
cargo run -- examples/fib.tribin

# Direct execution with release binary
./target/release/tri-vm examples/demo.tri
```

---

## 📁 Project Structure

```
tri-vm/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── trit.rs         # Trit type and operations
│   ├── tryte.rs        # Tryte type and arithmetic
│   ├── instruction.rs  # Instruction set & encoding
│   ├── vm.rs           # Virtual Machine core
│   ├── assembler.rs    # Assembler (Lexer → Parser → Codegen)
│   └── main.rs         # CLI binary
├── docs/
│   ├── DESIGN.md       # Full architecture documentation
│   ├── ROADMAP.md      # Development roadmap
│   ├── SYSTEM_ACCEPTANCE.md  # Acceptance criteria
│   └── TEST_CASES.md   # Test case design
├── examples/
│   ├── hello.tri       # Hello World example
│   ├── fib.tri         # Fibonacci sequence
│   ├── ternary.tri     # Balanced ternary arithmetic
│   ├── subroutine.tri  # CALL/RET demonstration
│   ├── dot_product.tri # MAC dot product
│   ├── demo.tri        # Complete feature showcase
│   ├── tnn.tri         # 4→4→1 TNN inference
│   └── tnn_ref.py      # Python reference for TNN
├── AGENTS.md          # AI agent guide
├── opencode.json
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## 📊 Development Progress

| Phase | Component | Tests | Status |
|-------|-----------|-------|--------|
| v0.1 | Trit | 11 | ✅ Complete |
| v0.2 | Tryte | 24 | ✅ Complete |
| v0.3 | Instruction | 22 | ✅ Complete |
| v0.4 | VM Core | 79 | ✅ Complete |
| v0.5 | Syscall + CLI | 10 | ✅ Complete |
| v0.6 | Assembler | 38 | ✅ Complete |
| v0.7 | AI Demo (TNN) | 6 | ✅ Complete |
| **Total** | | **190** | ✅ All Passing |

---

## 🧪 Example: Fibonacci

```asm
; fib(5) → R1=5, R2=8
  LDI  R1, 0        ; a = 0
  LDI  R2, 1        ; b = 1
  LDI  R4, 5        ; counter = N
  LDI  R5, 0        ; zero register

loop:
  BZ   R4, done     ; if counter == 0 → done
  ADD  R3, R1, R2   ; tmp = a + b
  ADD  R1, R2, R5   ; a = b
  ADD  R2, R3, R5   ; b = tmp
  ADDI R4, R4, -1   ; counter--
  JMP  loop

done:
  HALT
```

---

## 🤖 AI Demonstration

### Demo (all-instruction showcase)

```bash
cargo run -- examples/demo.tri
```

Output:
```
=== TriVM Demo ===

1.Arith=12           ← (3+5)×2-4
2.Div=3 r=1          ← 10÷3 remainder 1
3.MAC=32             ← Dot product [1,2,3]·[4,5,6]
4.Loop: 3 2 1        ← BZ/JMP control flow
5.Fact=24            ← CALL/RET factorial(4)
6.Mem=99             ← ST/LD memory operations
7.Tern=1TTT0         ← Balanced ternary output (42 in ternary)
8.Ovf=364 1          ← Overflow saturation + R7 flag
=== Done ===
```

### TNN (Ternary Neural Network)

```bash
cargo run -- examples/tnn.tri
```

Output: `1`

A 4→4→1 balanced ternary neural network performing inference (input `[1,1,T,T]` → output `1`). Weights are hardcoded in the `.data` section. The Python reference `examples/tnn_ref.py` validates all 8 test vectors.

---

## ✅ System Acceptance

| Scenario | Validation | Status |
|----------|------------|--------|
| S1 Fibonacci | Tryte arithmetic, instruction encoding, VM loop | ✅ Pass |
| S2 Hello World | Memory layout, strings, I/O | ✅ Pass |
| S3 Balanced Ternary | 0t prefix, ternary display | ✅ Pass |
| S4 MAC Primitive | Multiply-Accumulate for neural networks | ✅ Pass |
| S5 Subroutine | CALL/RET, stack operations | ✅ Pass |
| S6 Overflow | Saturation, R7 flag | ✅ Pass |
| S7 TNN Demo | 4→4→1 TNN inference, Python reference | ✅ Pass |

---

## 📚 Documentation

- [Design Document](docs/DESIGN.md) — Complete architecture reference
- [Development Roadmap](docs/ROADMAP.md) — Phased development plan
- [System Acceptance](docs/SYSTEM_ACCEPTANCE.md) — Full acceptance criteria
- [Test Cases](docs/TEST_CASES.md) — Test case specifications

---

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Submit issues
- Create pull requests
- Add new features
- Improve documentation

---

## 📝 AI-Generated Content Notice

Parts of this project were assisted by AI, including:
- Architecture design documentation
- Development roadmap
- Test case specifications
- This README file

All AI-generated content has been manually reviewed and validated.

---

## ⚠️ Disclaimer

This project is for **educational and experimental purposes only**. It explores balanced ternary computing architecture and is not intended for production use.

---

## 📄 License

MIT License - See [LICENSE](LICENSE) for details

---

## 🔗 Links

- **GitHub**: [https://github.com/marong103317-ops/tri-vm](https://github.com/marong103317-ops/tri-vm)
- **Release**: [https://github.com/marong103317-ops/tri-vm/releases](https://github.com/marong103317-ops/tri-vm/releases)

**Tags**: #ternary #vm #rust #ai #neuralnetwork #machinelearning #computerarchitecture #assembler #trit #tryte #ternarycomputing #alternativecomputing