use crate::trit::Trit::{self, NegOne, One, Zero};
use core::fmt;

pub const TRYTE_MIN: i16 = -364;
pub const TRYTE_MAX: i16 = 364;

/// 6-trit 平衡三进制字，范围 -364 ~ 364
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tryte(i16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TryteError {
    Overflow(Tryte),
    DivisionByZero,
}

impl Tryte {
    pub const MIN: Tryte = Tryte(-364);
    pub const MAX: Tryte = Tryte(364);
    pub const ZERO: Tryte = Tryte(0);
    pub const ONE: Tryte = Tryte(1);
    pub const NEG_ONE: Tryte = Tryte(-1);

    /// 从 i16 构造，范围外返回 None
    pub fn from_i16(v: i16) -> Option<Self> {
        if v < TRYTE_MIN || v > TRYTE_MAX {
            None
        } else {
            Some(Tryte(v))
        }
    }

    /// 从 [Trit; 6] 构造，永不失败
    pub fn from_trits(ts: [Trit; 6]) -> Self {
        let mut v: i16 = 0;
        let mut mul: i16 = 1;
        for &t in &ts {
            v += t.to_i8() as i16 * mul;
            mul *= 3;
        }
        Tryte(v)
    }

    /// 从 12-bit 编码构造，遇到 11 编码返回 None
    pub fn from_bits(bits: u16) -> Option<Self> {
        let mut ts = [Zero; 6];
        for i in 0..6 {
            let byte = ((bits >> (i * 2)) & 0b11) as u8;
            ts[i] = Trit::from_bits(byte)?;
        }
        Some(Tryte::from_trits(ts))
    }

    /// 转为 i16
    pub fn to_i16(self) -> i16 {
        self.0
    }

    /// 转为 [Trit; 6]，低位在前
    pub fn to_trits(self) -> [Trit; 6] {
        let mut remaining = self.0;
        let mut ts = [Zero; 6];
        for i in 0..6 {
            let r = remaining % 3;
            // 将 r (0, 1, 2) 映射到平衡三进制
            // 在平衡三进制中，我们想要值 0, 1, -1
            // remaining % 3 在 Rust 中给 -2..2
            // 但我们需要处理负数
            let t = if r == 2 || r == -1 {
                // 2 → -1 (借位)
                remaining = (remaining - (-1)) / 3;
                NegOne
            } else if r == -2 || r == 1 {
                // 1 → 1
                remaining = (remaining - 1) / 3;
                One
            } else {
                // r == 0
                remaining /= 3;
                Zero
            };
            ts[i] = t;
        }
        ts
    }

    /// 转为 12-bit 编码
    pub fn to_bits(self) -> u16 {
        let ts = self.to_trits();
        let mut bits: u16 = 0;
        for i in 0..6 {
            bits |= (ts[i].to_bits() as u16) << (i * 2);
        }
        bits
    }

    /// 加法，溢出饱和
    pub fn add(self, rhs: Self) -> Result<Self, TryteError> {
        let sum = self.0 as i32 + rhs.0 as i32;
        if sum > TRYTE_MAX as i32 {
            Err(TryteError::Overflow(Tryte(TRYTE_MAX)))
        } else if sum < TRYTE_MIN as i32 {
            Err(TryteError::Overflow(Tryte(TRYTE_MIN)))
        } else {
            Ok(Tryte(sum as i16))
        }
    }

    /// 减法，溢出饱和
    pub fn sub(self, rhs: Self) -> Result<Self, TryteError> {
        self.add(Tryte(-rhs.0))
    }

    /// 乘法，返回 (高 6 trits, 低 6 trits)，永不溢出
    pub fn mul_wide(self, rhs: Self) -> (Self, Self) {
        let product = self.0 as i32 * rhs.0 as i32;
        let lo_val = product % 729;
        let lo = if lo_val > 364 {
            lo_val - 729
        } else if lo_val < -364 {
            lo_val + 729
        } else {
            lo_val
        };
        let hi_val = (product - lo) / 729;
        (Tryte(hi_val as i16), Tryte(lo as i16))
    }

    /// 除法，向零取整。除零返回 Err
    pub fn div(self, rhs: Self) -> Result<Self, TryteError> {
        if rhs.0 == 0 {
            return Err(TryteError::DivisionByZero);
        }
        // Rust 整数除法截断向零，与平衡三进制一致
        let quotient = self.0 / rhs.0;
        // quotient 可能在范围外（理论上不会，因为被除数/除数 ≤ 被除数）
        // 但安全起见做饱和
        if quotient > TRYTE_MAX {
            Ok(Tryte(TRYTE_MAX))
        } else if quotient < TRYTE_MIN {
            Ok(Tryte(TRYTE_MIN))
        } else {
            Ok(Tryte(quotient))
        }
    }

    /// 取余，符号同被除数。除零返回 Err
    pub fn rem(self, rhs: Self) -> Result<Self, TryteError> {
        if rhs.0 == 0 {
            return Err(TryteError::DivisionByZero);
        }
        Ok(Tryte(self.0 % rhs.0))
    }

    /// 取反，永不失败
    pub fn neg(self) -> Self {
        Tryte(-self.0)
    }
}

impl fmt::Display for Tryte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ts = self.to_trits();
        // 跳过前导零，但保留至少一位
        let mut started = false;
        for &t in ts.iter().rev() {
            if t != Zero || started {
                write!(f, "{}", t)?;
                started = true;
            }
        }
        if !started {
            write!(f, "0")?;
        }
        Ok(())
    }
}

/// 从平衡三进制字符串解析，如 "1T0" = 6
impl std::str::FromStr for Tryte {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err("empty string".to_string());
        }

        let mut ts = [Zero; 6];
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        if len > 6 {
            return Err(format!("too many trits: {}", len));
        }

        for (i, &c) in chars.iter().enumerate() {
            let t = match c {
                'T' | 't' => NegOne,
                '0' => Zero,
                '1' => One,
                _ => return Err(format!("invalid char '{}'", c)),
            };
            // 字符串高位在前，数组低位在前，所以反转
            ts[len - 1 - i] = t;
        }
        Ok(Tryte::from_trits(ts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 构造 ──

    #[test]
    fn test_from_i16_valid() {
        assert_eq!(Tryte::from_i16(0).unwrap().to_i16(), 0);
        assert_eq!(Tryte::from_i16(364).unwrap().to_i16(), 364);
        assert_eq!(Tryte::from_i16(-364).unwrap().to_i16(), -364);
    }

    #[test]
    fn test_from_i16_invalid() {
        assert_eq!(Tryte::from_i16(365), None);
        assert_eq!(Tryte::from_i16(-365), None);
    }

    #[test]
    fn test_from_trits() {
        assert_eq!(Tryte::from_trits([Zero; 6]).to_i16(), 0);
        assert_eq!(Tryte::from_trits([One, Zero, Zero, Zero, Zero, Zero]).to_i16(), 1);
        assert_eq!(Tryte::from_trits([NegOne, Zero, Zero, Zero, Zero, Zero]).to_i16(), -1);
        // 0t1T0 = 1*9 + (-1)*3 + 0 = 6, 低位在 ts[0]
        assert_eq!(Tryte::from_trits([Zero, NegOne, One, Zero, Zero, Zero]).to_i16(), 6);
    }

    // ── 位编码往返 ──

    #[test]
    fn test_bits_roundtrip() {
        let cases = [-364, -1, 0, 1, 42, 364];
        for &v in &cases {
            let t = Tryte::from_i16(v).unwrap();
            let bits = t.to_bits();
            let back = Tryte::from_bits(bits).unwrap();
            assert_eq!(t, back, "bits roundtrip failed for {}", v);
        }
    }

    #[test]
    fn test_from_bits_rejects_11() {
        // 所有位都是 11 (=3)
        assert_eq!(Tryte::from_bits(0b111111111111), None);
        // 只有一位是 11
        assert_eq!(Tryte::from_bits(0b11), None);
    }

    // ── 加法 ──

    #[test]
    fn test_add_ok() {
        assert_eq!(Tryte::from_i16(1).unwrap().add(Tryte::from_i16(2).unwrap()).unwrap().to_i16(), 3);
        assert_eq!(Tryte::from_i16(-1).unwrap().add(Tryte::from_i16(1).unwrap()).unwrap().to_i16(), 0);
        assert_eq!(Tryte::from_i16(100).unwrap().add(Tryte::from_i16(200).unwrap()).unwrap().to_i16(), 300);
    }

    #[test]
    fn test_add_overflow_pos() {
        let r = Tryte::from_i16(364).unwrap().add(Tryte::from_i16(1).unwrap());
        assert_eq!(r, Err(TryteError::Overflow(Tryte(364))));
    }

    #[test]
    fn test_add_overflow_neg() {
        let r = Tryte::from_i16(-364).unwrap().add(Tryte::from_i16(-1).unwrap());
        assert_eq!(r, Err(TryteError::Overflow(Tryte(-364))));
    }

    // ── 减法 ──

    #[test]
    fn test_sub_ok() {
        assert_eq!(Tryte::from_i16(5).unwrap().sub(Tryte::from_i16(3).unwrap()).unwrap().to_i16(), 2);
        assert_eq!(Tryte::from_i16(0).unwrap().sub(Tryte::from_i16(1).unwrap()).unwrap().to_i16(), -1);
    }

    #[test]
    fn test_sub_overflow_neg() {
        let r = Tryte::from_i16(-364).unwrap().sub(Tryte::from_i16(1).unwrap());
        assert_eq!(r, Err(TryteError::Overflow(Tryte(-364))));
    }

    // ── 取反 ──

    #[test]
    fn test_neg() {
        assert_eq!(Tryte::from_i16(5).unwrap().neg().to_i16(), -5);
        assert_eq!(Tryte::from_i16(-364).unwrap().neg().to_i16(), 364);
        assert_eq!(Tryte::from_i16(0).unwrap().neg().to_i16(), 0);
    }

    // ── 乘法 ──

    #[test]
    fn test_mul_wide_small() {
        let a = Tryte::from_i16(3).unwrap();
        let b = Tryte::from_i16(5).unwrap();
        let (hi, lo) = a.mul_wide(b);
        assert_eq!(hi.to_i16(), 0);
        assert_eq!(lo.to_i16(), 15);
    }

    #[test]
    fn test_mul_wide_overflow_into_hi() {
        // 50 * 50 = 2500, 超过 364，需要高位
        let a = Tryte::from_i16(50).unwrap();
        let b = Tryte::from_i16(50).unwrap();
        let (hi, lo) = a.mul_wide(b);
        // 2500 = hi * 729 + lo, lo ∈ [-364, 364]
        // 2500 / 729 ≈ 3.43
        let reconstructed = hi.to_i16() as i32 * 729 + lo.to_i16() as i32;
        assert_eq!(reconstructed, 2500);
    }

    #[test]
    fn test_mul_wide_neg() {
        let a = Tryte::from_i16(-5).unwrap();
        let b = Tryte::from_i16(3).unwrap();
        let (hi, lo) = a.mul_wide(b);
        let reconstructed = hi.to_i16() as i32 * 729 + lo.to_i16() as i32;
        assert_eq!(reconstructed, -15);
    }

    #[test]
    fn test_mul_wide_max() {
        // max * max = 364 * 364 = 132496
        let a = Tryte::from_i16(364).unwrap();
        let b = Tryte::from_i16(364).unwrap();
        let (hi, lo) = a.mul_wide(b);
        let reconstructed = hi.to_i16() as i32 * 729 + lo.to_i16() as i32;
        assert_eq!(reconstructed, 132496);
        // 验证 lo 在范围内
        assert!(lo.to_i16() >= -364 && lo.to_i16() <= 364);
    }

    // ── 除法 ──

    #[test]
    fn test_div_ok() {
        assert_eq!(Tryte::from_i16(10).unwrap().div(Tryte::from_i16(3).unwrap()).unwrap().to_i16(), 3);
        assert_eq!(Tryte::from_i16(-10).unwrap().div(Tryte::from_i16(3).unwrap()).unwrap().to_i16(), -3);
        assert_eq!(Tryte::from_i16(10).unwrap().div(Tryte::from_i16(-3).unwrap()).unwrap().to_i16(), -3);
    }

    #[test]
    fn test_div_by_zero() {
        let a = Tryte::from_i16(5).unwrap();
        let b = Tryte::from_i16(0).unwrap();
        assert_eq!(a.div(b), Err(TryteError::DivisionByZero));
    }

    // ── 取余 ──

    #[test]
    fn test_rem_ok() {
        assert_eq!(Tryte::from_i16(10).unwrap().rem(Tryte::from_i16(3).unwrap()).unwrap().to_i16(), 1);
        assert_eq!(Tryte::from_i16(-10).unwrap().rem(Tryte::from_i16(3).unwrap()).unwrap().to_i16(), -1);
    }

    // ── 文本解析 ──

    #[test]
    fn test_from_str() {
        assert_eq!("0".parse::<Tryte>().unwrap().to_i16(), 0);
        assert_eq!("1".parse::<Tryte>().unwrap().to_i16(), 1);
        assert_eq!("T".parse::<Tryte>().unwrap().to_i16(), -1);
        assert_eq!("1T0".parse::<Tryte>().unwrap().to_i16(), 6);
        assert_eq!("T1".parse::<Tryte>().unwrap().to_i16(), -2);
        assert_eq!("10T".parse::<Tryte>().unwrap().to_i16(), 8);
    }

    #[test]
    fn test_from_str_empty() {
        assert!("".parse::<Tryte>().is_err());
    }

    #[test]
    fn test_from_str_invalid_char() {
        assert!("2".parse::<Tryte>().is_err());
        assert!("abc".parse::<Tryte>().is_err());
    }

    #[test]
    fn test_from_str_too_long() {
        assert!("1T01T0".parse::<Tryte>().is_ok());
        assert!("1T01T01".parse::<Tryte>().is_err());
    }

    // ── Display ──

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Tryte::from_i16(0).unwrap()), "0");
        assert_eq!(format!("{}", Tryte::from_i16(6).unwrap()), "1T0");
        assert_eq!(format!("{}", Tryte::from_i16(-2).unwrap()), "T1");
        assert_eq!(format!("{}", Tryte::from_i16(8).unwrap()), "10T");
        assert_eq!(format!("{}", Tryte::from_i16(-364).unwrap()), "TTTTTT");
    }

    // ── to_trits / from_trits 往返 ──

    #[test]
    fn test_trits_roundtrip() {
        let cases = [-364, -100, -1, 0, 1, 42, 100, 364];
        for &v in &cases {
            let t = Tryte::from_i16(v).unwrap();
            let ts = t.to_trits();
            let back = Tryte::from_trits(ts);
            assert_eq!(t, back, "trits roundtrip failed for {}", v);
        }
    }
}
