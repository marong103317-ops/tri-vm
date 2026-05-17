# TriVM — 平衡三进制虚拟机 (v7)

> 目标：在二进制硬件上用 Rust 实现一台最小平衡三进制虚拟机，证明三进制计算可以在二进制机器上跑通，并在此基础上探索三进制 AI 的可能性。
>
> 当前状态：Trit → Tryte → Instruction → VM → Assembler → TNN Demo 全链路完成。（190 测试全过）

---

## 1 Trit

平衡三进制的基本单位，取值 T (-1)、0、1。

```rust
enum Trit { NegOne, Zero, One }
```

| 运算 | 规则 |
|------|------|
| 取反 | T↔1, 0→0 |
| 加法 | `a+b` → (和, 进位)，-2→(1,T), -1→(T,0), 0→(0,0), 1→(1,0), 2→(T,1) |
| 减法 | `a-b = a+(-b)` |
| 乘法 | 真值表：T×T=1, T×1=T, 其余为 0 |

编码：2-bit 二进制，00=T, 01=0, 10=1, 11=非法。

---

## 2 Tryte

6 个 trit = 1 个字，范围 -364 ~ 364。

```rust
struct Tryte(i16);  // 内部用 i16 存储代数价值，构造时校验范围
```

| 运算 | 结果 |
|------|------|
| add/sub | 溢出饱和，返回 `Result` |
| mul | 返回 (hi: Tryte, lo: Tryte) 双字 12 trits |
| div | 除零返回 `Err` |
| neg | 永不失败（范围对称） |

构造方式：

| 方法 | 说明 |
|------|------|
| `from_i16` | 从 i16 构造，校验 -364~364 |
| `from_trits([Trit;6])` | 从 trit 数组构造 |
| `from_bits(u16)` | 从 12-bit 编码构造，校验 11 |
| `from_str("1T0")` | 从平衡三进制字符串解析 |
| `to_i16` | 转 i16 |
| `to_trits` | 转 trit 数组 |
| `to_bits` | 转 12-bit 编码 |
| `Display` | 输出平衡三进制字符串 |

---

## 3 寄存器

| 名称 | 宽度 | 用途 |
|------|------|------|
| R0–R7 | 6 trits | 通用（R7 溢出标志约定） |
| PC | 12 trits | 程序计数器 |
| SP | 12 trits | 栈指针 |

CMP 直接写通用寄存器，无专用 FLAGS。

---

## 4 内存

- tryte 寻址，59049 trytes（3^10）
- Rust 中表示为 `Vec<Tryte>` 或 `Vec<i16>`
- 地址 0 保留

---

## 5 指令集

指令宽度固定 2 tryte = 12 trits。第一 trit 定格式：

```
0 → Format R: 算术/逻辑/比较, 3 个寄存器操作数
1 → Format I: 立即数/内存, 寄存器 + 立即数
T → Format C: 控制流
```

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
I-a (2 寄存器 + 窄立即数): | 1 | op 2t | rd 3t | rs 3t | imm 3t |
I-b (1 寄存器 + 宽立即数): | 1 | op 2t | rd 3t | imm 6t |
```

| op | 子格式 | 指令 | 语义 |
|----|--------|------|------|
| 00 | I-b | `LDI Rd, imm` | Rd = imm (-364~364) |
| 0T | I-a | `ADDI Rd, Rs, imm` | Rd = Rs + imm (-13~13) |
| 01 | I-a | `MULI Rd, Rs, imm` | Rd = Rs × imm |
| 10 | I-a | `LD Rd, [Rs + imm]` | Rd = mem[Rs + imm] |
| 1T | I-a | `ST Rs, [Rd + imm]` | mem[Rd + imm] = Rs |
| 11 | —   | 保留 | — |
| T0 | —   | 保留 | — |
| TT | —   | 保留 | — |
| T1 | —   | 保留 | — |

### Format C

```
| T | op 2t | operand 9t |
```

| op | 指令 | 语义 |
|----|------|------|
| 00 | `JMP offset` | PC += offset_6t (±364 trytes) |
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

## 6 溢出

| 条件 | 行为 |
|------|------|
| 算术溢出 | 饱和 + R7=sgn(方向) |
| 除零 | 结果=0 + R7=T |
| 栈溢出/地址越界 | HALT |

> **注意**：MAC/ADDI/MULI/ADD/SUB 等指令都会写 `R7` 作为溢出标志，因此 **R7 不能在 MAC 循环中用作持久指针**。建议使用 R5 做地址指针（R5 不会被任何指令作为副效应写入）。

---

## 7 数字字面量

| 格式 | 示例 | 说明 |
|------|------|------|
| 裸数字 | `42` | 十进制 |
| 负数字 | `-5` | 负十进制 |
| 0t 前缀 | `0t1T0` | 平衡三进制 |
| 0x 前缀 | `0x2A` | 十六进制 |
| 0b 前缀 | `0b101010` | 二进制 |
| 裸 T | `T` | 平衡三进制 -1 |

---

## 8 内存布局

VM 初始内存布局（加载二进制时预分配）：

```
地址 0     保留
256        .text
20000      .data
40000      堆
55000      栈 (向下)
59048
```

> **注意**：汇编器输出为连续二进制，从地址 0 开始依次排放，不分段。汇编程序可在 `.text` 和 `.data` 间任意切换，地址连续增长。对于小程序（<182 条指令），所有标签地址仍在 Tryte 范围 ±364 内，可直接用 `LDI` 加载。

---

## 9 汇编器

### 9.1 公共 API

```rust
pub fn assemble(source: &str) -> Result<Vec<Tryte>, AssembleError>;
```

输入 `.tri` 源码 → 输出 `Vec<Tryte>` 可直接载入 VM 内存。

### 9.2 三阶段架构

```
源码 (.tri) → Lexer → [Token] → Parser → [ParsedLine] → Codegen → [Tryte]
```

| 阶段 | 输入 | 输出 | 说明 |
|------|------|------|------|
| Lexer | `&str` | `Vec<Token>` | 词法分析，识别助记符/寄存器/数字/标签/伪指令/注释 |
| Parser | `Vec<Token>` | `Vec<ParsedLine>` | 语法分析，标签绑定、指令校验、段追踪 |
| Codegen | `Vec<ParsedLine>` | `Vec<Tryte>` | 两趟：第一遍建符号表，第二遍发射指令并解析标签 |

### 9.3 数字字面量

参见第 7 节。

注意：裸 `T` 或 `t`（无 `0t` 前缀）解析为 -1（平衡三进制一位数字）。纯数字如 `10` 始终解析为十进制（10），不会误判为三进制。

### 9.4 伪指令

| 伪指令 | 语法 | 说明 |
|--------|------|------|
| `.text` | `.text` | 切换至代码段（默认） |
| `.data` | `.data` | 切换至数据段 |
| `.word v1, v2, ...` | `.word 1, -3, 0tT1` | 发射若干 Tryte 立即数 |
| `.asciiz s` | `.asciiz "Hello"` | 发射 null 结尾字符串 |

### 9.5 标签

- 裸标签（独占一行）：绑定到 **下一行指令/伪指令**
- 行内标签（`label:` 在指令前）：绑定到该指令
- 连续裸标签报错（防止静默覆盖）
- 文件末尾裸标签报错

### 9.6 分支偏移计算

```asm
  JMP target  ; offset = target - PC（当前行地址，非 PC+2）
  BZ  R0, lbl ; offset = lbl - PC
```

偏移量为相对于当前 PC 的差值（PC 内增前）。

---

## 10 项目结构

```
src/
├── lib.rs
├── main.rs          ✅ v0.5 CLI
├── trit.rs          ✅ v0.1 完成
├── tryte.rs         ✅ v0.2 完成
├── instruction.rs   ✅ v0.3 完成
├── vm.rs            ✅ v0.4 + v0.5 Syscall
├── assembler.rs     ✅ v0.6 Assembler
examples/
├── tnn.tri          ✅ v0.7 TNN 推理
├── tnn_ref.py       ✅ v0.7 Python 参考
```

---

## 11 实现顺序

| 阶段 | 内容 | 验证 |
|------|------|------|
| v0.1 | Trit | 11 测试 ✅ |
| v0.2 | Tryte | 24 测试 ✅ |
| v0.3 | Instruction | 22 测试 ✅ |
| v0.4 | VM + Fibonacci | 79 测试 ✅ |
| v0.5 | Syscall + CLI | 10 测试 ✅ |
| v0.6 | Assembler | 38 测试 ✅ |
| v0.7 | AI demo (TNN) | 6 测试 ✅ (共 190) |
| v0.8+ | 未来方向 | 待定 |
