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

## 阶段 1：VM 核心 — v0.4

- [ ] **VM 状态** — `src/vm.rs`
  - [ ] 寄存器文件 `[Tryte; 8]`
  - [ ] PC（12 trits，用 `i32` 或两个 Tryte 表示）
  - [ ] SP（12 trits）
  - [ ] 内存 `Vec<Tryte>`（59049 容量）
  - [ ] 内存读写方法（含越界检查）
  - **验收**：能创建 VM 实例，读写内存不越界

- [ ] **指令执行：Format R**
  - [ ] NOP
  - [ ] ADD / SUB / MUL
  - [ ] DIV / MOD（除零设 R7=T，结果=0）
  - [ ] MAC（乘累加）
  - [ ] CMP（返回 -1/0/1）
  - [ ] SGN（符号函数）
  - **验收**：每个 Format R 指令至少一个单元测试覆盖语义正确

- [ ] **指令执行：Format I**
  - [ ] LDI（载入 6-trit 立即数）
  - [ ] ADDI / MULI（寄存器 + 窄立即数）
  - [ ] LD（从内存载入）
  - [ ] ST（存入内存）
  - **验收**：每个 Format I 指令至少一个单元测试覆盖语义正确

- [ ] **指令执行：Format C**
  - [ ] JMP（PC 相对跳转）
  - [ ] JMP Rs（间接跳转）
  - [ ] BZ / BN / BP（条件跳转）
  - [ ] CALL / RET（子程序调用）
  - [ ] SYSCALL（仅桩，后续实现）
  - [ ] HALT（停机）
  - **验收**：每个 Format C 指令至少一个单元测试覆盖语义正确

- [ ] **主循环：fetch-decode-execute**
  - [ ] 从内存 PC 位置读取 2 tryte
  - [ ] decode → Instruction
  - [ ] execute → 更新状态
  - [ ] 溢出捕获（R7 标志 + 饱和），不 panic
  - **验收**：三步循环集成测试通过

- [ ] **Fibonacci 集成测试**
  - [ ] 手工编码 Fibonacci 程序的内存镜像（N→Fib(N)）
  - [ ] 在 VM 上运行，验证结果正确
  - **验收**：`fib(0)=0, fib(1)=1, fib(7)=13`

---

## 阶段 2：I/O 与 CLI — v0.5

- [ ] **系统调用实现** — `src/syscall.rs`
  - [ ] EXIT（终止 VM，返回退出码）
  - [ ] PRINT_T（平衡三进制输出）
  - [ ] PRINT_D（十进制输出）
  - [ ] PRINT_C（ASCII 字符输出）
  - [ ] PRINT_S（字符串输出）
  - **验收**：Fibonacci 程序能通过 SYSCALL 输出结果

- [ ] **CLI binary** — `src/main.rs`
  - [ ] 读取 .tri 二进制文件
  - [ ] 加载到 VM 内存
  - [ ] 运行 VM
  - [ ] 打印输出
  - **验收**：`cargo run -- examples/fib.tri` 输出正确结果

---

## 阶段 3：汇编器 — v0.6

- [ ] **Lexer**
  - [ ] 识别指令助记符、寄存器、数字（十进制/0t/0x/0b）
  - [ ] 识别标签、注释、伪指令
  - **验收**：Fibonacci 源码 token 化通过

- [ ] **Parser**
  - [ ] 标签解析 → 符号表
  - [ ] 指令解析 → 操作数校验
  - [ ] 伪指令（.data, .text, .word, .asciiz）
  - **验收**：Fibonacci 源码 AST 构建通过

- [ ] **Codegen**
  - [ ] 指令 → Tryte 对
  - [ ] 标签 → 地址重定位
  - [ ] 输出 .tri 二进制
  - **验收**：Fibonacci.tri → 内存镜像 → VM 执行 → 正确结果

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
v0.3  │ Instruction     │  21   │ ✅
v0.4  │ VM 核心         │   ~   │ ⬜  ← 现在在这里
v0.5  │ Syscall + CLI   │   ~   │ ⬜
v0.6  │ Assembler       │   ~   │ ⬜
v0.7  │ AI demo         │   ~   │ ⬜
```
