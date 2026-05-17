use core::fmt;

/// 平衡三进制的一位：T (-1), 0, 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trit {
    NegOne,
    Zero,
    One,
}

impl Trit {
    /// 从 i8 构造，非 -1/0/1 返回 None
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::NegOne),
            0 => Some(Trit::Zero),
            1 => Some(Trit::One),
            _ => None,
        }
    }

    /// 转为 i8
    pub fn to_i8(self) -> i8 {
        match self {
            Trit::NegOne => -1,
            Trit::Zero => 0,
            Trit::One => 1,
        }
    }

    /// 取反：T↔1, 0→0
    pub fn neg(self) -> Self {
        match self {
            Trit::NegOne => Trit::One,
            Trit::Zero => Trit::Zero,
            Trit::One => Trit::NegOne,
        }
    }

    /// 平衡三进制加法。
    /// 返回 (和, 进位)，进位为 T/0/1。
    pub fn add(self, rhs: Trit) -> (Self, Self) {
        let s = self.to_i8() + rhs.to_i8();
        match s {
            -2 => (Trit::One, Trit::NegOne),
            -1 => (Trit::NegOne, Trit::Zero),
            0 => (Trit::Zero, Trit::Zero),
            1 => (Trit::One, Trit::Zero),
            2 => (Trit::NegOne, Trit::One),
            _ => unreachable!(),
        }
    }

    /// a - b = a + (-b)
    pub fn sub(self, rhs: Trit) -> (Self, Self) {
        self.add(rhs.neg())
    }

    /// 乘法真值表
    pub fn mul(self, rhs: Trit) -> Self {
        match (self, rhs) {
            (Trit::NegOne, Trit::NegOne) => Trit::One,
            (Trit::NegOne, Trit::Zero) => Trit::Zero,
            (Trit::NegOne, Trit::One) => Trit::NegOne,
            (Trit::Zero, _) => Trit::Zero,
            (Trit::One, Trit::NegOne) => Trit::NegOne,
            (Trit::One, Trit::Zero) => Trit::Zero,
            (Trit::One, Trit::One) => Trit::One,
        }
    }

    /// 从 2-bit 二进制编码解码：00→T, 01→0, 10→1。11 返回 None。
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits & 0b11 {
            0b00 => Some(Trit::NegOne),
            0b01 => Some(Trit::Zero),
            0b10 => Some(Trit::One),
            _ => None,
        }
    }

    /// 编码为 2-bit 二进制：T→00, 0→01, 1→10
    pub fn to_bits(self) -> u8 {
        match self {
            Trit::NegOne => 0b00,
            Trit::Zero => 0b01,
            Trit::One => 0b10,
        }
    }
}

impl fmt::Display for Trit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trit::NegOne => write!(f, "T"),
            Trit::Zero => write!(f, "0"),
            Trit::One => write!(f, "1"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 验收标准：构造 ──

    #[test]
    fn test_from_i8_valid() {
        assert_eq!(Trit::from_i8(-1), Some(Trit::NegOne));
        assert_eq!(Trit::from_i8(0), Some(Trit::Zero));
        assert_eq!(Trit::from_i8(1), Some(Trit::One));
    }

    #[test]
    fn test_from_i8_invalid() {
        assert_eq!(Trit::from_i8(2), None);
        assert_eq!(Trit::from_i8(-2), None);
    }

    // ── 验收标准：转换 ──

    #[test]
    fn test_to_i8_roundtrip() {
        assert_eq!(Trit::NegOne.to_i8(), -1);
        assert_eq!(Trit::Zero.to_i8(), 0);
        assert_eq!(Trit::One.to_i8(), 1);
    }

    // ── 验收标准：取反，两次恢复 ──

    #[test]
    fn test_neg() {
        assert_eq!(Trit::NegOne.neg(), Trit::One);
        assert_eq!(Trit::Zero.neg(), Trit::Zero);
        assert_eq!(Trit::One.neg(), Trit::NegOne);
    }

    #[test]
    fn test_neg_twice_is_identity() {
        let all = [Trit::NegOne, Trit::Zero, Trit::One];
        for &t in &all {
            assert_eq!(t.neg().neg(), t, "neg(neg({})) != {}", t, t);
        }
    }

    // ── 验收标准：加法真值表穷举 ──

    #[test]
    fn test_add_exhaustive() {
        let cases = [
            // (lhs, rhs, (sum, carry))
            (Trit::NegOne, Trit::NegOne, (Trit::One, Trit::NegOne)),
            (Trit::NegOne, Trit::Zero,  (Trit::NegOne, Trit::Zero)),
            (Trit::NegOne, Trit::One,   (Trit::Zero, Trit::Zero)),
            (Trit::Zero,  Trit::NegOne, (Trit::NegOne, Trit::Zero)),
            (Trit::Zero,  Trit::Zero,   (Trit::Zero, Trit::Zero)),
            (Trit::Zero,  Trit::One,    (Trit::One, Trit::Zero)),
            (Trit::One,   Trit::NegOne, (Trit::Zero, Trit::Zero)),
            (Trit::One,   Trit::Zero,   (Trit::One, Trit::Zero)),
            (Trit::One,   Trit::One,    (Trit::NegOne, Trit::One)),
        ];
        for (lhs, rhs, expected) in cases {
            assert_eq!(lhs.add(rhs), expected, "{} + {}", lhs, rhs);
        }
    }

    // ── 验收标准：减法 ──

    #[test]
    fn test_sub_exhaustive() {
        let cases = [
            (Trit::NegOne, Trit::NegOne, (Trit::Zero, Trit::Zero)),
            (Trit::NegOne, Trit::Zero,   (Trit::NegOne, Trit::Zero)),
            (Trit::NegOne, Trit::One,    (Trit::One, Trit::NegOne)),
            (Trit::Zero,  Trit::NegOne,  (Trit::One, Trit::Zero)),
            (Trit::Zero,  Trit::Zero,    (Trit::Zero, Trit::Zero)),
            (Trit::Zero,  Trit::One,     (Trit::NegOne, Trit::Zero)),
            (Trit::One,   Trit::NegOne,  (Trit::NegOne, Trit::One)),
            (Trit::One,   Trit::Zero,    (Trit::One, Trit::Zero)),
            (Trit::One,   Trit::One,     (Trit::Zero, Trit::Zero)),
        ];
        for (lhs, rhs, expected) in cases {
            assert_eq!(lhs.sub(rhs), expected, "{} - {}", lhs, rhs);
        }
    }

    // ── 验收标准：乘法真值表穷举 ──

    #[test]
    fn test_mul_exhaustive() {
        let cases = [
            (Trit::NegOne, Trit::NegOne, Trit::One),
            (Trit::NegOne, Trit::Zero,  Trit::Zero),
            (Trit::NegOne, Trit::One,   Trit::NegOne),
            (Trit::Zero,  Trit::NegOne, Trit::Zero),
            (Trit::Zero,  Trit::Zero,   Trit::Zero),
            (Trit::Zero,  Trit::One,    Trit::Zero),
            (Trit::One,   Trit::NegOne, Trit::NegOne),
            (Trit::One,   Trit::Zero,   Trit::Zero),
            (Trit::One,   Trit::One,    Trit::One),
        ];
        for (lhs, rhs, expected) in cases {
            assert_eq!(lhs.mul(rhs), expected, "{} × {}", lhs, rhs);
        }
    }

    // ── 验收标准：bits 编解码无损 ──

    #[test]
    fn test_bits_roundtrip() {
        let all = [Trit::NegOne, Trit::Zero, Trit::One];
        for &t in &all {
            let bits = t.to_bits();
            let decoded = Trit::from_bits(bits);
            assert_eq!(decoded, Some(t), "bits roundtrip failed for {}", t);
        }
    }

    #[test]
    fn test_from_bits_rejects_11() {
        assert_eq!(Trit::from_bits(0b11), None);
    }

    // ── 验收标准：Display ──

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Trit::NegOne), "T");
        assert_eq!(format!("{}", Trit::Zero), "0");
        assert_eq!(format!("{}", Trit::One), "1");
    }
}
