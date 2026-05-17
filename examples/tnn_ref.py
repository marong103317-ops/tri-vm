#!/usr/bin/env python3
"""TriVM v0.7 — Balanced Ternary Neural Network reference implementation.

4→4→1 fully connected network with SGN activation.
Weights designed by hand to produce varied outputs across test cases.
"""

T = -1

def sgn(x):
    return 1 if x > 0 else (T if x < 0 else 0)

def layer(x, W, b):
    """Fully connected layer: out = sgn(W @ x + b)"""
    n_out = len(W)
    out = []
    for i in range(n_out):
        s = b[i]
        for j in range(len(x)):
            s += W[i][j] * x[j]
        out.append(sgn(s))
    return out

def format_ternary(v):
    """Format a value for TriVM .data: T for -1, otherwise decimal."""
    return "T" if v == -1 else str(v)

def gen_asm_weights(name, W, b):
    """Generate TriVM .data section for weights."""
    lines = []
    for i, row in enumerate(W):
        for j, w in enumerate(row):
            lines.append(f"{name}_w{i}{j}: .word {format_ternary(w)}")
    for i, bi in enumerate(b):
        lines.append(f"{name}_b{i}: .word {format_ternary(bi)}")
    return lines

# ─── Network weights (4×4→4→1) ───

W1 = [
    [1, 0, T, 0],
    [0, 1, 0, T],
    [T, 0, 1, 0],
    [0, T, 0, 1],
]
b1 = [0, 0, 0, 0]

W2 = [[1, 1, T, T]]
b2 = [0]

# ─── Test cases ───

test_inputs = [
    [1, 1, T, T],
    [T, T, 1, 1],
    [1, 0, T, 0],
    [0, 0, 0, 0],
    [1, 1, 1, 1],
    [T, T, T, T],
    [1, T, 1, T],
    [0, 1, 0, T],
]

print("=" * 60)
print("TriVM v0.7 — TNN Reference")
print("=" * 60)

print("\nWeights (Layer 1, 4×4):")
for row in W1:
    print(f"  [{', '.join(format_ternary(w) for w in row)}]")
print(f"b1 = [{', '.join(format_ternary(b) for b in b1)}]")

print("\nWeights (Layer 2, 1×4):")
print(f"  [{', '.join(format_ternary(w) for w in W2[0])}]")
print(f"b2 = [{format_ternary(b2[0])}]")

print("\n" + "-" * 60)
print(f"{'Input':<24} {'Hidden':<24} {'Output':<8}")
print("-" * 60)

# Run all test cases
for inp in test_inputs:
    h = layer(inp, W1, b1)
    out = layer(h, W2, b2)[0]

    inp_str = f"[{', '.join(format_ternary(v) for v in inp)}]"
    h_str = f"[{', '.join(format_ternary(v) for v in h)}]"
    print(f"{inp_str:<24} {h_str:<24} {format_ternary(out):<8}")

print("\n" + "=" * 60)

# Export TriVM .data weights
print("\n# TriVM .data export:")
print(".data")
for line in gen_asm_weights("ly1", W1, b1):
    print(f"  {line}")
for line in gen_asm_weights("ly2", W2, b2):
    print(f"  {line}")
