# TriVM — 平衡三进制虚拟机

![GitHub stars](https://img.shields.io/github/stars/marong103317-ops/tri-vm?style=social)
![GitHub forks](https://img.shields.io/github/forks/marong103317-ops/tri-vm?style=social)
![GitHub license](https://img.shields.io/github/license/marong103317-ops/tri-vm)
![GitHub release](https://img.shields.io/github/v/release/marong103317-ops/tri-vm)
![Rust](https://img.shields.io/badge/language-Rust-orange)

> 一个用 Rust 在二进制硬件上实现的最小**平衡三进制虚拟机**。证明三进制计算可以在二进制机器上运行，并探索**三进制 AI** 和**神经网络**的可能性。

## 🚀 特性

- **三进制计算**: 原生支持平衡三进制算术（-1, 0, 1）
- **完整虚拟机**: 23 条指令的完整指令集
- **汇编器**: 内置 `.tri` 源文件汇编器
- **命令行界面**: 直接从源码或编译后的二进制运行三进制程序
- **AI 原语**: 用于神经网络的 MAC（乘累加）操作
- **190 个测试**: 全面的测试覆盖

## 🎯 关键词与标签

`ternary`, `balanced-ternary`, `virtual-machine`, `VM`, `Rust`, `AI`, `neural-network`, `machine-learning`, `computer-architecture`, `Trit`, `Tryte`, `MAC`, `三进制`, `虚拟机`, `神经网络`, `人工智能`

---

## 🏗️ 核心概念

### Trit — 三进制位

平衡三进制的基本单位，有三种状态：**T** (-1)、**0** (零)、**1** (一)。

| 运算 | 规则 |
|------|------|
| 取反 | T↔1, 0→0 |
| 加法 | 完整真值表，含进位 |
| 乘法 | T×T=1, T×1=T, 其余为 0 |

**编码**: 2-bit 二进制（00=T, 01=0, 10=1, 11=无效）

### Tryte — 三进制字

6 个 trit = 1 个 tryte，范围 **-364 到 364**。内部用 `i16` 存储。

### 寄存器

| 寄存器 | 宽度 | 用途 |
|--------|------|------|
| R0-R7 | 6 trits | 通用寄存器（R7 = 溢出标志） |
| PC | 12 trits | 程序计数器 |
| SP | 12 trits | 栈指针 |

### 内存布局

```
地址 0     保留
256        .text 段
20000      .data 段
40000      堆
55000      栈（向下生长）
59048      最大地址 (3^10 - 1)
```

---

## 📋 指令集

固定宽度 2 tryte = 12 trits。第一个 trit 决定格式：

- **0** → Format R: 算术/逻辑/比较（3 个寄存器操作数）
- **1** → Format I: 立即数/内存（寄存器 + 立即数）
- **T** → Format C: 控制流

### Format R 指令

| 操作码 | 助记符 | 语义 |
|--------|--------|------|
| 00 | `NOP` | 空操作 |
| 01 | `ADD` | Rd = Rs + Rt |
| 0T | `SUB` | Rd = Rs - Rt |
| 10 | `MUL` | Rd = Rs × Rt |
| 1T | `DIV` | Rd = Rs / Rt |
| 11 | `MOD` | Rd = Rs % Rt |
| T0 | `MAC` | Rd += Rs × Rt（乘累加） |
| TT | `CMP` | Rd = sgn(Rs - Rt) |
| T1 | `SGN` | Rd = sgn(Rs) |

### Format I 指令

| 操作码 | 助记符 | 语义 |
|--------|--------|------|
| 00 | `LDI Rd, imm` | Rd = 立即数 (-364..364) |
| 0T | `ADDI Rd, Rs, imm` | Rd = Rs + 立即数 (-13..13) |
| 01 | `MULI Rd, Rs, imm` | Rd = Rs × 立即数 |
| 10 | `LD Rd, [Rs+imm]` | Rd = 内存[Rs+imm] |
| 1T | `ST Rs, [Rd+imm]` | 内存[Rd+imm] = Rs |

### Format C 指令

| 操作码 | 助记符 | 语义 |
|--------|--------|------|
| 00 | `JMP offset` | PC += 偏移 |
| 0T | `JMP Rs` | PC = Rs（间接跳转） |
| 01 | `BZ Rs, offset` | 如果 Rs == 0 则跳转 |
| 10 | `BN Rs, offset` | 如果 Rs < 0 则跳转 |
| 1T | `BP Rs, offset` | 如果 Rs > 0 则跳转 |
| 11 | `CALL offset` | 压栈 PC+2，PC += 偏移 |
| T0 | `RET` | 出栈到 PC |
| TT | `SYSCALL Rs` | 系统调用 |
| T1 | `HALT` | 停止执行 |

### 系统调用 (SYSCALL Rs)

| 功能码 | 名称 | 描述 |
|--------|------|------|
| 1 | `EXIT` | 终止，R0 = 退出码 |
| 2 | `PRINT_T` | 以三进制输出 R0 |
| 3 | `PRINT_D` | 以十进制输出 R0 |
| 4 | `PRINT_C` | 以 ASCII 字符输出 R0 |
| 7 | `PRINT_S` | 输出以 null 结尾的字符串 |

---

## ⚡ 快速开始

```bash
# 构建虚拟机
cargo build --release

# 运行所有测试（190 个）
cargo test

# 运行汇编源文件
cargo run -- examples/hello.tri

# 运行预编译的二进制文件
cargo run -- examples/fib.tribin

# 使用发布版本直接执行
./target/release/tri-vm examples/demo.tri
```

---

## 📁 项目结构

```
tri-vm/
├── src/
│   ├── lib.rs          # 库入口
│   ├── trit.rs         # Trit 类型和运算
│   ├── tryte.rs        # Tryte 类型和算术
│   ├── instruction.rs  # 指令集和编码
│   ├── vm.rs           # 虚拟机核心
│   ├── assembler.rs    # 汇编器（词法分析 → 语法分析 → 代码生成）
│   └── main.rs         # 命令行界面
├── docs/
│   ├── DESIGN.md       # 完整架构文档
│   ├── ROADMAP.md      # 开发路线图
│   ├── SYSTEM_ACCEPTANCE.md  # 验收标准
│   └── TEST_CASES.md   # 测试用例设计
├── examples/
│   ├── hello.tri       # Hello World 示例
│   ├── fib.tri         # 斐波那契数列
│   ├── ternary.tri     # 平衡三进制算术
│   ├── subroutine.tri  # CALL/RET 演示
│   ├── dot_product.tri # MAC 点积
│   ├── demo.tri        # 完整功能展示
│   ├── tnn.tri         # 4→4→1 TNN 推理
│   └── tnn_ref.py      # TNN Python 参考实现
├── AGENTS.md          # AI 代理指南
├── opencode.json
├── Cargo.toml
├── LICENSE
├── README.md           # 英文版本
└── README-zh.md        # 中文版本
```

---

## 📊 开发进度

| 阶段 | 组件 | 测试数 | 状态 |
|------|------|--------|------|
| v0.1 | Trit | 11 | ✅ 已完成 |
| v0.2 | Tryte | 24 | ✅ 已完成 |
| v0.3 | Instruction | 22 | ✅ 已完成 |
| v0.4 | VM 核心 | 79 | ✅ 已完成 |
| v0.5 | Syscall + CLI | 10 | ✅ 已完成 |
| v0.6 | Assembler | 38 | ✅ 已完成 |
| v0.7 | AI 演示 (TNN) | 6 | ✅ 已完成 |
| **总计** | | **190** | ✅ 全部通过 |

---

## 🧪 示例：斐波那契数列

```asm
; fib(5) → R1=5, R2=8
  LDI  R1, 0        ; a = 0
  LDI  R2, 1        ; b = 1
  LDI  R4, 5        ; counter = N
  LDI  R5, 0        ; 零寄存器

loop:
  BZ   R4, done     ; 如果 counter == 0 → 结束
  ADD  R3, R1, R2   ; tmp = a + b
  ADD  R1, R2, R5   ; a = b
  ADD  R2, R3, R5   ; b = tmp
  ADDI R4, R4, -1   ; counter--
  JMP  loop

done:
  HALT
```

---

## 🤖 AI 演示

### 综合演示（全指令展示）

```bash
cargo run -- examples/demo.tri
```

输出：
```
=== TriVM Demo ===

1.Arith=12           ← (3+5)×2-4
2.Div=3 r=1          ← 10÷3 余 1
3.MAC=32             ← 点积 [1,2,3]·[4,5,6]
4.Loop: 3 2 1        ← BZ/JMP 控制流
5.Fact=24            ← CALL/RET 阶乘(4)
6.Mem=99             ← ST/LD 内存操作
7.Tern=1TTT0         ← 平衡三进制输出（42 的三进制表示）
8.Ovf=364 1          ← 溢出饱和 + R7 标志
=== Done ===
```

### TNN（三进制神经网络）

```bash
cargo run -- examples/tnn.tri
```

输出：`1`

一个 4→4→1 平衡三进制神经网络推理程序（输入 `[1,1,T,T]` → 输出 `1`）。权重硬编码在 `.data` 段中。Python 参考实现 `examples/tnn_ref.py` 验证全部 8 个测试向量。

---

## ✅ 系统验收

| 场景 | 验证内容 | 状态 |
|------|---------|------|
| S1 Fibonacci | Tryte 算术、指令编码、VM 循环 | ✅ 通过 |
| S2 Hello World | 内存布局、字符串、I/O | ✅ 通过 |
| S3 平衡三进制运算 | 0t 前缀、三进制显示 | ✅ 通过 |
| S4 MAC 原语 | 乘累加，用于神经网络 | ✅ 通过 |
| S5 子程序调用 | CALL/RET、栈操作 | ✅ 通过 |
| S6 溢出检测 | 溢出饱和、R7 标志 | ✅ 通过 |
| S7 TNN 推理 | 4→4→1 TNN 推理，Python 参考验证 | ✅ 通过 |

---

## 📚 文档

- [设计文档](docs/DESIGN.md) — 完整架构参考
- [开发路线图](docs/ROADMAP.md) — 分阶段开发计划
- [系统验收标准](docs/SYSTEM_ACCEPTANCE.md) — 全链路验收标准
- [测试用例](docs/TEST_CASES.md) — 测试用例规范

---

## 🤝 贡献

欢迎贡献！您可以：
- 提交 issues
- 创建 pull requests
- 添加新功能
- 改进文档

---

## 📝 AI 生成内容声明

本项目的部分内容由 AI 辅助生成，包括：
- 架构设计文档
- 开发路线图
- 测试用例规范
- 本 README 文件

所有 AI 生成内容均已通过人工审阅和验证。

---

## ⚠️ 免责声明

本项目仅供**教育和实验目的**使用。它探索平衡三进制计算架构，不适合生产环境使用。

---

## 📄 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE)

---

## 🔗 链接

- **GitHub**: [https://github.com/marong103317-ops/tri-vm](https://github.com/marong103317-ops/tri-vm)
- **发布**: [https://github.com/marong103317-ops/tri-vm/releases](https://github.com/marong103317-ops/tri-vm/releases)

**标签**: #ternary #vm #rust #ai #neuralnetwork #三进制 #虚拟机 #神经网络 #人工智能