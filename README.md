# TriVM — 平衡三进制虚拟机

> 在二进制硬件上用 Rust 实现一台最小平衡三进制虚拟机，证明三进制计算可以在二进制机器上跑通，并在此基础上探索三进制 AI 的可能性。

---

## 核心概念

### Trit — 基本单位

平衡三进制的基本信息单位，取值 **T** (−1)、**0**、**1**。

| 运算 | 规则 |
|------|------|
| 取反 | T↔1, 0→0 |
| 加法 | `a+b` → (和, 进位)，−2→(1,T), −1→(T,0), 0→(0,0), 1→(1,0), 2→(T,1) |
| 乘法 | 真值表：T×T=1, T×1=T, 其余为 0 |

编码：2-bit 二进制，00=T, 01=0, 10=1, 11=非法。

### Tryte — 数据字

6 个 trit = 1 个字，范围 **−364 ~ 364**。内部用 `i16` 存储代数价值。

### 寄存器

| 名称 | 宽度 | 用途 |
|------|------|------|
| R0–R7 | 6 trits | 通用（R7 溢出标志约定） |
| PC | 12 trits | 程序计数器 |
| SP | 12 trits | 栈指针 |

### 内存

- tryte 寻址，59049 trytes（3¹⁰）
- 地址 0 保留

```
地址 0     保留
256        .text
20000      .data
40000      堆
55000      栈 (向下)
59048
```

---

## 指令集

指令宽度固定 2 tryte = 12 trits。第一 trit 定格式：

- **0** → Format R：算术/逻辑/比较，3 个寄存器操作数
- **1** → Format I：立即数/内存，寄存器 + 立即数
- **T** → Format C：控制流

### Format R

```
| 0 | op 2t | rd 3t | rs 3t | rt 3t |
```

| op | 指令 | 语义 |
|----|------|------|
| 00 | `NOP` | — |
| 01 | `ADD` | Rd = Rs + Rt |
| 0T | `SUB` | Rd = Rs − Rt |
| 10 | `MUL` | Rd = Rs × Rt |
| 1T | `DIV` | Rd = Rs / Rt（向零取整）|
| 11 | `MOD` | Rd = Rs % Rt |
| T0 | `MAC` | Rd += Rs × Rt |
| TT | `CMP` | Rd = sgn(Rs − Rt) |
| T1 | `SGN` | Rd = sgn(Rs) |

### Format I

```
I-a: | 1 | op 2t | rd 3t | rs 3t | imm 3t |
I-b: | 1 | op 2t | rd 3t | imm 6t |
```

| op | 指令 | 语义 |
|----|------|------|
| 00 | `LDI Rd, imm` | Rd = imm (−364~364) |
| 0T | `ADDI Rd, Rs, imm` | Rd = Rs + imm (−13~13) |
| 01 | `MULI Rd, Rs, imm` | Rd = Rs × imm |
| 10 | `LD Rd, [Rs + imm]` | Rd = mem[Rs + imm] |
| 1T | `ST Rs, [Rd + imm]` | mem[Rd + imm] = Rs |

### Format C

```
| T | op 2t | operand 9t |
```

| op | 指令 | 语义 |
|----|------|------|
| 00 | `JMP offset` | PC += offset (±364 trytes) |
| 0T | `JMP Rs` | PC = Rs（间接跳转） |
| 01 | `BZ Rs, offset` | if Rs==0: PC += offset |
| 10 | `BN Rs, offset` | if Rs<0: PC += offset |
| 1T | `BP Rs, offset` | if Rs>0: PC += offset |
| 11 | `CALL offset` | 压 PC+2，PC += offset |
| T0 | `RET` | 弹出 PC |
| TT | `SYSCALL Rs` | 系统调用 |
| T1 | `HALT` | 停机 |

### 系统调用

| 功能码 | 名称 | 说明 |
|--------|------|------|
| 1 | `EXIT` | 终止，R0=退出码 |
| 2 | `PRINT_T` | 三进制输出 R0 |
| 3 | `PRINT_D` | 十进制输出 R0 |
| 4 | `PRINT_C` | 输出 ASCII 字符 R0 |
| 7 | `PRINT_S` | 输出字符串 (R0=地址) |

---

## 溢出与异常

| 条件 | 行为 |
|------|------|
| 算术溢出 | 饱和 + R7=sgn(方向) |
| 除零 | 结果=0 + R7=T |
| 栈溢出/地址越界 | HALT |

---

## 快速开始

```bash
# 构建
cargo build

# 运行测试
cargo test

# 运行（需要 .tri 二进制文件）
cargo run -- <file.tribin>
```

---

## 项目结构

```
src/
├── lib.rs
├── trit.rs         # Trit 类型与运算
├── tryte.rs        # Tryte 类型与算术
├── instruction.rs  # 指令集编解码
├── vm.rs           # 虚拟机核心
└── main.rs         # CLI 入口
```

---

## 开发进度

| 阶段 | 内容 | 测试数 | 状态 |
|------|------|--------|------|
| v0.1 | Trit | 11 | ✅ 已完成 |
| v0.2 | Tryte | 24 | ✅ 已完成 |
| v0.3 | Instruction | 21 | ✅ 已完成 |
| v0.4 | VM 核心 | ~93 | ⬜ 进行中 |
| v0.5 | Syscall + CLI | — | ⬜ |
| v0.6 | Assembler | — | ⬜ |
| v0.7 | AI demo | — | ⬜ |

---

## 示例：Fibonacci

```asm
; fib(N) → 输出到控制台
.text
  LDI  R0, N        ; N = 目标序号
  LDI  R1, 0        ; a = 0
  LDI  R2, 1        ; b = 1

loop:
  BZ   R0, done     ; if N == 0 → done
  BP   R0, calc     ; if N > 0 → continue
  JMP  done

calc:
  MOV  R3, R2       ; tmp = b
  ADD  R2, R1, R2   ; b = a + b
  MOV  R1, R3       ; a = tmp
  ADDI R0, R0, T    ; N -= 1
  JMP  loop

done:
  MOV  R0, R1
  SYSCALL 3         ; PRINT_D
  SYSCALL 1         ; EXIT
```

---

## 系统验收场景

| 场景 | 验证内容 |
|------|---------|
| S1 Fibonacci | Tryte 算术、Inst 编解码、VM 循环 |
| S2 Hello World | 内存布局、字符串、I/O |
| S3 平衡三进制运算 | 0t 前缀、三进制显示 |
| S4 MAC 原语 | 乘法、MAC 累加 |
| S5 子程序调用 | CALL/RET、栈操作 |
| S6 溢出检测 | 溢出饱和、R7 标志 |

通过全部 6 个场景 = 系统验收通过。

---

## 文档

- [设计文档](docs/DESIGN.md) — 完整架构与指令集定义
- [开发路线图](docs/ROADMAP.md) — 分阶段开发计划与验收标准
- [系统验收标准](docs/SYSTEM_ACCEPTANCE.md) — 全链路验收场景
- [测试用例](docs/TEST_CASES.md) — v0.4 VM 核心测试用例设计

---

## AI 生成声明

本项目中的部分代码与文档由 AI 辅助生成，包括但不限于：

- 项目设计文档（`docs/DESIGN.md`）
- 开发路线图（`docs/ROADMAP.md`）
- 系统验收标准（`docs/SYSTEM_ACCEPTANCE.md`）
- 测试用例设计（`docs/TEST_CASES.md`）
- 本 README 文件

所有 AI 生成内容均已经过人工审阅和验证，但可能仍存在错误或遗漏。使用者应自行判断和验证相关内容的正确性。

## 免责声明

本项目仅供学习与实验目的，属于对平衡三进制计算架构的探索性实现。

- 本项目按"原样"（AS IS）提供，不作任何明示或暗示的保证，包括但不限于适销性、特定用途适用性和非侵权性。
- 作者不对因使用本项目代码或文档而产生的任何直接、间接、附带、特殊或后果性损害承担责任。
- 本项目中的虚拟机实现未经生产环境验证，不应用于任何关键系统或业务场景。
- 平衡三进制计算仍属实验性领域，本项目中的设计决策不一定代表最优方案。

---

## License

MIT
