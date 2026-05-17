# TriVM 开发路线图

## 图例

- `[ ]` 待做
- `[x]` 已完成

---

## 阶段 0：基础类型 ✅

- [x] **Trit** — `src/trit.rs`
  - [x] enum + 穷举运算（add/sub/mul/neg）
  - [x] bits 编解码（00=T, 01=0, 10=1, 11 拒斥）
  - [x] Display
  - **验收**：11 测试全过

- [x] **Tryte** — `src/tryte.rs`
  - [x] 6-trit 字（i16 存储，范围 -364~364）
  - [x] 算术（add/sub/neg/div/rem/mul_wide），溢出返回 Result
  - [x] 构造（from_i16, from_trits, from_bits, FromStr）
  - [x] 转换（to_i16, to_trits, to_bits, Display）
  - [x] 边界（±364 溢出饱和，除零报错，11 编码拒斥，-364取反不溢出）
  - **验收**：24 测试全过

- [x] **Instruction** — `src/instruction.rs`
  - [x] Reg 类型 + 8 种编码映射
  - [x] Instruction enum（23 种指令）
  - [x] encode: Inst → (Tryte, Tryte)
  - [x] decode: (Tryte, Tryte) → Inst
  - [x] 所有 23 种指令编解码往返正确
  - **验收**：21 测试全过

---

## 阶段 1：VM 核心 — v0.4 ✅

- [x] **VM 状态** — `src/vm.rs`
  - [x] 寄存器文件 `[Tryte; 8]`
  - [x] PC（12 trits，用 `usize` 表示）
  - [x] SP（12 trits）
  - [x] 内存 `Vec<Tryte>`（59049 容量）
  - [x] 内存读写方法（含越界检查 → HALT）
  - **验收**：能创建 VM 实例，读写内存不越界

- [x] **指令执行：Format R**
  - [x] NOP
  - [x] ADD / SUB / MUL
  - [x] DIV / MOD（除零设 R7=T，结果=0）
  - [x] MAC（乘累加）
  - [x] CMP（返回 -1/0/1）
  - [x] SGN（符号函数）
  - **验收**：28 个单元测试覆盖所有 Format R 指令 (R-01~R-74)

- [x] **指令执行：Format I**
  - [x] LDI（载入 6-trit 立即数）
  - [x] ADDI / MULI（寄存器 + 窄立即数）
  - [x] LD（从内存载入）
  - [x] ST（存入内存）
  - **验收**：21 个单元测试覆盖所有 Format I 指令 (I-01~I-42)

- [x] **指令执行：Format C**
  - [x] JMP（PC 相对跳转）
  - [x] JMP Rs（间接跳转）
  - [x] BZ / BN / BP（条件跳转）
  - [x] CALL / RET（子程序调用）
  - [x] SYSCALL（仅桩，后续实现）
  - [x] HALT（停机）
  - **验收**：19 个单元测试覆盖所有 Format C 指令 (C-01~C-71)

- [x] **主循环：fetch-decode-execute**
  - [x] 从内存 PC 位置读取 2 tryte
  - [x] decode → Instruction
  - [x] execute → 更新状态
  - [x] 溢出捕获（R7 标志 + 饱和），不 panic
  - **验收**：三步循环集成测试通过 (test_run_simple_program)

- [x] **Fibonacci 集成测试**
  - [x] 手工编码 Fibonacci 程序的内存镜像（N=5 → Fib(5)）
  - [x] 在 VM 上运行，验证 R1=5, R2=8
  - **验收**：`fib(5)=5, fib(6)=8`

---

## 阶段 2：I/O 与 CLI — v0.5 ✅

- [x] **系统调用实现** — `src/vm.rs`
  - [x] EXIT（终止 VM，返回退出码 R0）
  - [x] PRINT_T（平衡三进制输出 R0）
  - [x] PRINT_D（十进制输出 R0）
  - [x] PRINT_C（ASCII 字符输出 R0）
  - [x] PRINT_S（字符串输出，地址 R0，最长 256 字符）
  - **验收**：10 个 Syscall 测试全过 (S-01~S-50)

- [x] **CLI binary** — `src/main.rs`
  - [x] 读取 .tri 二进制文件（raw i16 LE）
  - [x] 加载到 VM 内存
  - [x] 运行 VM
  - [x] 打印输出 + 返回退出码
  - **验收**：`cargo run -- examples/fib.tribin` 输出正确结果

---

## 阶段 3：汇编器 — v0.6 ✅

- [x] **Lexer**
  - [x] 识别指令助记符、寄存器、数字（十进制/0t/0x/0b/裸T）
  - [x] 识别标签、注释、伪指令（.text/.data/.word/.asciiz）
  - [x] 字符串字面量解析
  - **验收**：Fibonacci 源码 token 化通过

- [x] **Parser**
  - [x] 标签解析 → 符号表（支持连续标签报错）
  - [x] 指令解析 → 操作数校验
  - [x] 伪指令（.data, .text, .word, .asciiz）
  - [x] HALT/RET 禁止携带操作数
  - **验收**：Fibonacci 源码 AST 构建通过

- [x] **Codegen**
  - [x] 两趟：第一趟建符号表，第二趟发射 Tryte 并解析标签
  - [x] 标签 → 地址重定位（相对偏移）
  - [x] 纯数字不做三进制误判
  - [x] 文件末尾裸标签报错
  - **验收**：Fibonacci.tri → 内存镜像 → VM 执行 → 正确结果
  - **验收**：所有 5 个示例（hello/fib/ternary/subroutine/dot_product）端到端通过

- [x] **回归测试**（v0.6.1）
  - [x] 纯数字不做三进制误判（`10` ≠ 三进制 `10`）
  - [x] 连续裸标签报错
  - [x] HALT/RET 拒绝操作数
  - [x] 文件末尾裸标签报错
  - **验收**：+8 个回归测试，全部通过

---

## 阶段 4：AI 演示 — v0.7

- [ ] **TNN 层原语**
  - [ ] 矩阵乘（MAC 循环）
  - [ ] SGN 激活
  - [ ] 全连接层
  - **验收**：能在 TriVM 上运行一个 4×4×1 的小网络

- [ ] **AI 示例程序**
  - [ ] 训练好的权重编码为 .data
  - [ ] 前向推理汇编代码
  - [ ] 输出分类结果
  - **验收**：输入→推理→输出，与 Python 参考实现比对一致

---

## 汇总

```
阶段  │ 内容            │ 测试数 │ 状态
──────┼─────────────────┼───────┼───────
v0.1  │ Trit            │  11   │ ✅
v0.2  │ Tryte           │  24   │ ✅
v0.3  │ Instruction     │  22   │ ✅
v0.4  │ VM 核心         │  79   │ ✅
v0.5  │ Syscall + CLI   │  10   │ ✅
v0.6  │ Assembler       │   38  │ ✅
v0.7  │ AI demo         │   ~   │ ⬜
──────┼─────────────────┼───────┼───────
合计  │                 │ 184   │ 全部通过
```
