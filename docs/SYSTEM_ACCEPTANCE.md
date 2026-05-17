# TriVM 系统验收标准

> 当前进展：**全部 7 个场景（S1~S7）均已实现汇编源码全链路通过**。
>
> v0.7 TNN Demo 完成：4→4→1 平衡三进制神经网络在 TriVM 上成功推理，与 Python 参考实现一致。
>
> CLI 自动检测输入是 `.tri` 还是 `.tribin`。

## 1 总则

系统验收的目标：**一个用 TriASM 编写的程序，经过汇编、加载、执行，最终输出正确结果**。

全链路：
```
.tri 源码 → 汇编器 → .tri 二进制 → VM 加载 → 执行 → 输出
```

每一层都可以独立验证，但系统验收要求全链路跑通。

---

## 2 验收场景

### S1: Fibonacci 计算

**输入**：计算第 N 个斐波那契数（当前通过手工编码内存镜像实现）

**手工编码程序**（Rust 测试 `test_fibonacci`）：
```asm
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

**验证矩阵**：

| N | R1 (fib(N)) | R2 (fib(N+1)) | 状态 |
|---|------------|---------------|------|
| 5 | 5          | 8             | ✅ 已通过 |

---

### S2: Hello World

**输入**：输出字符串

```asm
.text
  LDI   R0, msg
  LDI   R1, 7         ; PRINT_S
  SYSCALL R1
  LDI   R1, 1         ; EXIT
  SYSCALL R1

.data
msg:
  .asciiz "Hello, TriVM!"
```

**通过条件**：输出 `Hello, TriVM!`。

**状态**：✅ `cargo run -- examples/hello.tri` 输出正确。

---

### S3: 平衡三进制运算

**输入**：验证平衡三进制算术正确性

```asm
; 计算 0t1T0 + 0tT1 = ?
; 0t1T0 = 6, 0tT1 = -2, 6 + (-2) = 4 = 0t11
  LDI  R0, 0t1T0
  LDI  R1, 0tT1
  ADD  R0, R0, R1      ; R0 = 6 + (-2) = 4
  LDI  R1, 2           ; PRINT_T
  SYSCALL R1           ; → 应输出 "11"
  LDI  R1, 3           ; PRINT_D
  SYSCALL R1           ; → 应输出 "4"
  LDI  R1, 1           ; EXIT
  SYSCALL R1
```

**通过条件**：输出 `114`（三进制 11，紧接十进制 4）。

**状态**：✅ `cargo run -- examples/ternary.tri` 输出 `114`。

---

### S4: MAC 与 AI 原语

**输入**：两个向量的点积

```asm
; 计算 [1, T, 1] · [1, 1, T] = 1×1 + T×1 + 1×T = 1 + (-1) + (-1) = -1
.text
  LDI  R0, 0            ; sum = 0

  LDI  R1, 1
  LDI  R2, 1
  MAC  R0, R1, R2       ; sum += 1 × 1 = 1

  LDI  R1, T
  LDI  R2, 1
  MAC  R0, R1, R2       ; sum += T × 1 = -1 → 0

  LDI  R1, 1
  LDI  R2, T
  MAC  R0, R1, R2       ; sum += 1 × T = -1 → -1

  LDI  R1, 3            ; PRINT_D
  SYSCALL R1            ; → 应输出 "-1"
  LDI  R1, 1            ; EXIT
  SYSCALL R1
```

**通过条件**：输出 `-1`。

**状态**：✅ `cargo run -- examples/dot_product.tri` 输出 `-1`。

---

### S5: 子程序调用

**输入**：验证 CALL/RET 正确性

```asm
.text
  LDI   R0, 5
  CALL  square          ; R0 = square(5) = 25
  LDI   R1, 3
  SYSCALL R1
  LDI   R1, 1
  SYSCALL R1

square:
  MUL   R0, R0, R0
  RET
```

**通过条件**：输出 `25`。

**状态**：✅ `cargo run -- examples/subroutine.tri` 输出 `25`。

---

### S7: TNN（三进制神经网络）推理

**输入**：4→4→1 全连接网络，隐藏层 + 输出层各带 SGN 激活

```
输入 [4] → FC(4→4) → SGN → FC(4→1) → SGN → 输出 [1]
```

**权重配置**（详见 `examples/tnn.tri`）：
- Layer 1 权重矩阵 4×4，每神经元 4 权重 + 1 偏置（全 0）
- Layer 2 权重矩阵 1×4，偏置 0

**验证矩阵**：

| 输入 | 期望输出 | 状态 |
|------|---------|------|
| [1, 1, T, T] | 1 | ✅ `cargo run -- examples/tnn.tri` 输出 `1` |
| [0, 0, 0, 0] | 0 | ✅ 内联测试通过 |
| [1, 1, 1, 1] | 0 | ✅ 内联测试通过 |

Python 参考实现 `examples/tnn_ref.py` 覆盖全部 8 个测试向量。

**通过条件**：与 Python 参考实现输出一致。

**状态**：✅ 全链路通过。

---

### S6: 溢出检测

**输入**：验证溢出饱和机制

```asm
  LDI  R5, 0
  LDI  R0, 364
  LDI  R1, 1
  ADD  R0, R0, R1       ; 应饱和到 364
  LDI  R1, 3            ; PRINT_D
  SYSCALL R1            ; 输出 "364"
  ADD  R0, R7, R5       ; R7 = 溢出标志
  LDI  R1, 3
  SYSCALL R1            ; 输出 "1"（正溢出）
  LDI  R1, 1
  SYSCALL R1
```

**通过条件**：输出 `3641`（364 紧接 1）。

**状态**：✅ 验证通过（demo.tri 中包含此场景）。

---

## 3 分层验收映射

| 系统场景 | 覆盖的基础层 | 覆盖的指令 | 状态 |
|---------|-------------|-----------|------|
| S1 Fibonacci | Tryte 算术、Inst 编解码、VM 循环 | LDI, BZ, ADD, ADDI, JMP, HALT | ✅ 汇编全链路 |
| S2 Hello World | 内存布局、字符串、I/O | LDI, SYSCALL(PRINT_S/EXIT) | ✅ 汇编全链路 |
| S3 平衡三进制 | 0t 前缀、三进制显示 | LDI(0t), ADD, SYSCALL(PRINT_T/PRINT_D/EXIT) | ✅ 汇编全链路 |
| S4 MAC 原语 | 乘法、MAC | LDI, MAC, SYSCALL(PRINT_D/EXIT) | ✅ 汇编全链路 |
| S5 子程序 | 栈、返回地址 | CALL, RET, MUL, SYSCALL(PRINT_D/EXIT) | ✅ 汇编全链路 |
| S6 溢出 | 溢出饱和、R7 标志 | ADD 溢出边界, SYSCALL(PRINT_D/EXIT) | ✅ 汇编全链路 |
| S7 TNN | MAC 循环、SGN 激活、CALL/RET | LDI, MAC, ADDI, LD, ADD, SGN, CALL, RET, ST, SYSCALL | ✅ 汇编全链路 |

**通过全部 7 个场景 = 系统验收通过** ✅

---

## 4 执行方式

### 4.1 自动验证（CI）

每个场景对应一个集成测试：

```rust
#[test]
fn system_fibonacci_n7() {
    let bytecode = assemble(include_str!("fib.tri"));
    let mut vm = VM::new();
    vm.load(&bytecode);
    vm.run();
    assert_eq!(vm.output, "13");
}
```

### 4.2 手动验证（CLI）

CLI 自动检测文件后缀：

```bash
# 汇编源码 .tri → 自动汇编 + 执行
cargo run -- examples/hello.tri
# 期望输出: Hello, TriVM!

# 预编译二进制 .tribin → 直接加载 + 执行
cargo run -- examples/fib.tribin
# 期望输出: （无输出，程序 halt 或无 PRINT syscall）
```

### 4.3 分阶段验证（无汇编器时，已废弃）

汇编器已就绪。以下为历史参考：在没有汇编器的情况下，可以手动构造内存镜像，直接用 VM 加载：

```rust
// 手工编码 fib(7) 的内存镜像
let mem = vec![
    /* LDI R0, 7 */   encode(&Instruction::Ldi { rd: R(0), imm: t(7) }),
    /* LDI R1, 0 */   encode(&Instruction::Ldi { rd: R(1), imm: t(0) }),
    /* ... */
];
let mut vm = VM::new();
vm.load(&mem);
vm.run();
assert_eq!(vm.regs[1].to_i16(), 13);  // fib(7) = 13
```

---

## 5 不可验收项（本次范围外）

以下内容属于"有更好，但不影响系统验收通过"：

- VM 执行性能（周期计数）
- 调试器（断点、单步）
- AI 模型的训练过程（仅前向推理）
- 非 macOS 平台兼容性
