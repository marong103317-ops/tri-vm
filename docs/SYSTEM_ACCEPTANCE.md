# TriVM 系统验收标准

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

**输入**：计算第 N 个斐波那契数

**TriASM 源码**（`fib.tri`）：
```asm
; fib(N) → 输出到控制台
; N 通过 LDI 硬编码在源码中

.text
  LDI  R0, N        ; N = 目标序号
  LDI  R1, 0        ; a = 0
  LDI  R2, 1        ; b = 1

loop:
  BZ   R0, done     ; if N == 0 → a 已在 R1
  BP   R0, calc     ; if N > 0 → 继续
  JMP  done         ; 不应到达

calc:
  MOV  R3, R2       ; tmp = b
  ADD  R2, R1, R2   ; b = a + b
  MOV  R1, R3       ; a = tmp
  ADDI R0, R0, T    ; N -= 1
  JMP  loop

done:
  ; 将结果 R1 输出
  MOV  R0, R1
  SYSCALL 3         ; PRINT_D
  SYSCALL 1         ; EXIT
```

**验证矩阵**：

| N | 期望输出 |
|---|---------|
| 0 | 0       |
| 1 | 1       |
| 2 | 1       |
| 7 | 13      |
| 10 | 55     |

**通过条件**：汇编 → 加载 → 执行 → 输出与期望一致。

---

### S2: Hello World

**输入**：输出字符串

```asm
.text
  LDI   R0, msg
  SYSCALL 7         ; PRINT_S
  SYSCALL 1         ; EXIT

.data
msg:
  .asciiz "Hello, TriVM!"
```

**通过条件**：输出 `Hello, TriVM!`。

---

### S3: 平衡三进制运算

**输入**：验证平衡三进制算术正确性

```asm
; 计算 0t1T0 + 0tT1 = ?
; 0t1T0 = 6, 0tT1 = -2, 6 + (-2) = 4 = 0t11
  LDI  R0, 0t1T0
  LDI  R1, 0tT1
  ADD  R0, R0, R1      ; R0 = 6 + (-2) = 4
  SYSCALL 2             ; PRINT_T → 应输出 "11"
  SYSCALL 3             ; PRINT_D → 应输出 "4"
  SYSCALL 1
```

**通过条件**：输出 `114`（三进制 11，紧接十进制 4）。

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

  SYSCALL 3             ; PRINT_D → 应输出 "-1"
  SYSCALL 1
```

**通过条件**：输出 `-1`。

---

### S5: 子程序调用

**输入**：验证 CALL/RET 正确性

```asm
.text
  LDI   R0, 5
  CALL  square          ; R0 = square(5) = 25
  SYSCALL 3
  SYSCALL 1

square:
  MUL   R0, R0, R0
  RET
```

**通过条件**：输出 `25`。

---

### S6: 溢出检测

**输入**：验证溢出饱和机制

```asm
  LDI  R0, 364
  LDI  R1, 1
  ADD  R0, R0, R1       ; 应饱和到 364
  SYSCALL 3             ; 输出 "364"
  MOV  R0, R7           ; R7 = 溢出标志
  SYSCALL 3             ; 输出 "1"（正溢出）
  SYSCALL 1
```

**通过条件**：输出 `3641`（364 紧接 1）。

---

## 3 分层验收映射

| 系统场景 | 覆盖的基础层 | 覆盖的指令 |
|---------|-------------|-----------|
| S1 Fibonacci | Tryte 算术、Inst 编解码、VM 循环 | LDI, BZ, BP, MOV, ADD, ADDI, JMP, SYSCALL |
| S2 Hello World | 内存布局、字符串、I/O | LDI, SYSCALL(PRINT_S/EXIT), .asciiz |
| S3 平衡三进制 | 0t 前缀、三进制显示 | LDI(0t), ADD, SYSCALL(PRINT_T/PRINT_D) |
| S4 MAC 原语 | 乘法、MAC | LDI, MAC, SYSCALL |
| S5 子程序 | 栈、返回地址 | CALL, RET, MUL |
| S6 溢出 | 溢出饱和、R7 标志 | ADD 溢出边界 |

**通过全部 6 个场景 = 系统验收通过**。

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

```bash
# 汇编 + 执行
triasm examples/fib.tri -o fib.tribin
trivm fib.tribin
# 期望输出: 13
```

### 4.3 分阶段验证（无汇编器时）

在没有汇编器的情况下，可以手动构造内存镜像，直接用 VM 加载：

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

- 汇编器错误提示友好度
- VM 执行性能（周期计数）
- 调试器（断点、单步）
- AI 模型的训练过程（仅前向推理）
- 非 macOS 平台兼容性
