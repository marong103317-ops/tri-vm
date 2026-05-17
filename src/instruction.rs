use crate::trit::Trit::{self, NegOne, One, Zero};
use crate::tryte::Tryte;

/// 寄存器索引 0-7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reg(pub u8);

impl Reg {
    pub fn from_raw(raw: [Trit; 3]) -> Option<Self> {
        match raw {
            [Zero, Zero, Zero] => Some(Reg(0)),
            [One, Zero, Zero] => Some(Reg(1)),
            [NegOne, Zero, Zero] => Some(Reg(2)),
            [Zero, One, Zero] => Some(Reg(3)),
            [NegOne, One, Zero] => Some(Reg(4)),
            [One, One, Zero] => Some(Reg(5)),
            [Zero, NegOne, Zero] => Some(Reg(6)),
            [NegOne, NegOne, Zero] => Some(Reg(7)),
            _ => None,
        }
    }

    pub fn to_raw(self) -> [Trit; 3] {
        match self.0 {
            0 => [Zero, Zero, Zero],
            1 => [One, Zero, Zero],
            2 => [NegOne, Zero, Zero],
            3 => [Zero, One, Zero],
            4 => [NegOne, One, Zero],
            5 => [One, One, Zero],
            6 => [Zero, NegOne, Zero],
            7 => [NegOne, NegOne, Zero],
            _ => unreachable!(),
        }
    }

    pub fn idx(self) -> u8 {
        self.0
    }
}

/// 译码错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidRegister,
    ReservedOpcode,
}

/// 指令
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    // Format R
    Nop,
    Add { rd: Reg, rs: Reg, rt: Reg },
    Sub { rd: Reg, rs: Reg, rt: Reg },
    Mul { rd: Reg, rs: Reg, rt: Reg },
    Div { rd: Reg, rs: Reg, rt: Reg },
    Modulo { rd: Reg, rs: Reg, rt: Reg },
    Mac { rd: Reg, rs: Reg, rt: Reg },
    Cmp { rd: Reg, rs: Reg, rt: Reg },
    Sgn { rd: Reg, rs: Reg },

    // Format I-a (2 regs + 3t imm)
    AddI { rd: Reg, rs: Reg, imm: Tryte },
    MulI { rd: Reg, rs: Reg, imm: Tryte },
    Ld { rd: Reg, rs: Reg, imm: Tryte },
    St { rd: Reg, rs: Reg, imm: Tryte },

    // Format I-b (1 reg + 6t imm)
    Ldi { rd: Reg, imm: Tryte },

    // Format C
    Jmp { offset: Tryte },
    JmpR { rs: Reg },
    Bz { rs: Reg, offset: Tryte },
    Bn { rs: Reg, offset: Tryte },
    Bp { rs: Reg, offset: Tryte },
    Call { offset: Tryte },
    Ret,
    Syscall { rs: Reg },
    Halt,
}

// ── 底层辅助 ──

/// 从 12 trits 中提取 3 trits 的寄存器字段
fn extract_reg(ts: &[Trit; 12], start: usize) -> Option<Reg> {
    let raw = [ts[start], ts[start + 1], ts[start + 2]];
    Reg::from_raw(raw)
}

/// 将 3 trits 写回 12-trit 数组
fn write_reg(ts: &mut [Trit; 12], reg: Reg, start: usize) {
    let raw = reg.to_raw();
    ts[start] = raw[0];
    ts[start + 1] = raw[1];
    ts[start + 2] = raw[2];
}

// ── 公开 API ──

/// 从两个 Tryte 解码为指令（先高后低）
pub fn decode(hi: Tryte, lo: Tryte) -> Result<Instruction, DecodeError> {
    // 将两个 tryte 拼为 12 trits
    let hi_ts = hi.to_trits();  // 6 trits
    let lo_ts = lo.to_trits();  // 6 trits
    let mut ts = [Zero; 12];
    // hi 是高位 tryte → 对应 ts[6..12]
    // lo 是低位 tryte → 对应 ts[0..6]
    for i in 0..6 {
        ts[i] = lo_ts[i];
        ts[i + 6] = hi_ts[i];
    }

    let fmt = ts[11]; // 最高位 trit 决定格式
    match fmt {
        Zero => decode_format_r(&ts),
        One => decode_format_i(&ts),
        NegOne => decode_format_c(&ts),
    }
}

/// 将指令编码为两个 Tryte（先高后低）
pub fn encode(inst: &Instruction) -> (Tryte, Tryte) {
    let ts = match inst {
        // Format R
        Instruction::Nop => {
            [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero]
        }
        Instruction::Add { rd, rs, rt } => format_r(Zero, One, rd, rs, rt),
        Instruction::Sub { rd, rs, rt } => format_r(Zero, NegOne, rd, rs, rt),
        Instruction::Mul { rd, rs, rt } => format_r(One, Zero, rd, rs, rt),
        Instruction::Div { rd, rs, rt } => format_r(One, NegOne, rd, rs, rt),
        Instruction::Modulo { rd, rs, rt } => format_r(One, One, rd, rs, rt),
        Instruction::Mac { rd, rs, rt } => format_r(NegOne, Zero, rd, rs, rt),
        Instruction::Cmp { rd, rs, rt } => format_r(NegOne, NegOne, rd, rs, rt),
        Instruction::Sgn { rd, rs } => format_r(NegOne, One, rd, rs, &Reg(0)),
        // Format I-a
        Instruction::AddI { rd, rs, imm } => format_ia(Zero, NegOne, rd, rs, imm),
        Instruction::MulI { rd, rs, imm } => format_ia(Zero, One, rd, rs, imm),
        Instruction::Ld { rd, rs, imm } => format_ia(One, Zero, rd, rs, imm),
        Instruction::St { rd, rs, imm } => format_ia(One, NegOne, rd, rs, imm),
        // Format I-b
        Instruction::Ldi { rd, imm } => format_ib(Zero, Zero, rd, imm),
        // Format C
        Instruction::Jmp { offset } => format_c_offset(Zero, Zero, offset),
        Instruction::JmpR { rs } => format_c_reg(Zero, NegOne, rs),
        Instruction::Bz { rs, offset } => format_c_reg_offset(Zero, One, rs, offset),
        Instruction::Bn { rs, offset } => format_c_reg_offset(One, Zero, rs, offset),
        Instruction::Bp { rs, offset } => format_c_reg_offset(One, NegOne, rs, offset),
        Instruction::Call { offset } => format_c_offset(One, One, offset),
        Instruction::Ret => {
            let mut ts = [Zero; 12];
            ts[11] = NegOne; // fmt = T
            ts[10] = NegOne; // opcode = T0
            ts
        }
        Instruction::Syscall { rs } => format_c_reg(NegOne, NegOne, rs),
        Instruction::Halt => {
            let mut ts = [Zero; 12];
            ts[11] = NegOne; // fmt = T
            ts[10] = NegOne; // opcode = T1
            ts[9] = One;
            ts
        }
    };

    let hi_ts: [Trit; 6] = [
        ts[6], ts[7], ts[8], ts[9], ts[10], ts[11],
    ];
    let lo_ts: [Trit; 6] = [
        ts[0], ts[1], ts[2], ts[3], ts[4], ts[5],
    ];
    (Tryte::from_trits(hi_ts), Tryte::from_trits(lo_ts))
}

// ── 格式构造辅助函数 ──

/// Format R: | 0 | op 2t | rd 3t | rs 3t | rt 3t |
fn format_r(op_t2: Trit, op_t1: Trit, rd: &Reg, rs: &Reg, rt: &Reg) -> [Trit; 12] {
    let mut ts = [Zero; 12];
    ts[11] = Zero; // fmt = 0
    ts[10] = op_t2;
    ts[9] = op_t1;
    write_reg(&mut ts, *rd, 6);
    write_reg(&mut ts, *rs, 3);
    write_reg(&mut ts, *rt, 0);
    ts
}

/// Format I-a: | 1 | op 2t | rd 3t | rs 3t | imm 3t |
fn format_ia(op_t2: Trit, op_t1: Trit, rd: &Reg, rs: &Reg, imm: &Tryte) -> [Trit; 12] {
    let imm_trits = imm.to_trits(); // [3^0..3^5]
    let mut ts = [Zero; 12];
    ts[11] = One; // fmt = 1
    ts[10] = op_t2;
    ts[9] = op_t1;
    write_reg(&mut ts, *rd, 6);
    write_reg(&mut ts, *rs, 3);
    // imm 3t → 取低 3 trits
    ts[2] = imm_trits[2];
    ts[1] = imm_trits[1];
    ts[0] = imm_trits[0];
    ts
}

/// Format I-b: | 1 | op 2t | rd 3t | imm 6t |
fn format_ib(op_t2: Trit, op_t1: Trit, rd: &Reg, imm: &Tryte) -> [Trit; 12] {
    let imm_trits = imm.to_trits();
    let mut ts = [Zero; 12];
    ts[11] = One;
    ts[10] = op_t2;
    ts[9] = op_t1;
    write_reg(&mut ts, *rd, 6);
    // imm 6t
    for i in 0..6 {
        ts[i] = imm_trits[i];
    }
    ts
}

/// Format C: | T | op 2t | reg 3t | offset 6t |
fn format_c_reg_offset(op_t2: Trit, op_t1: Trit, rs: &Reg, offset: &Tryte) -> [Trit; 12] {
    let off_ts = offset.to_trits();
    let mut ts = [Zero; 12];
    ts[11] = NegOne; // fmt = T
    ts[10] = op_t2;
    ts[9] = op_t1;
    write_reg(&mut ts, *rs, 6);
    for i in 0..6 {
        ts[i] = off_ts[i];
    }
    ts
}

/// Format C: | T | op 2t | offset 6t | (无寄存器)
fn format_c_offset(op_t2: Trit, op_t1: Trit, offset: &Tryte) -> [Trit; 12] {
    let off_ts = offset.to_trits();
    let mut ts = [Zero; 12];
    ts[11] = NegOne;
    ts[10] = op_t2;
    ts[9] = op_t1;
    for i in 0..6 {
        ts[i] = off_ts[i];
    }
    ts
}

/// Format C: | T | op 2t | reg 3t | (无偏移)
fn format_c_reg(op_t2: Trit, op_t1: Trit, rs: &Reg) -> [Trit; 12] {
    let mut ts = [Zero; 12];
    ts[11] = NegOne;
    ts[10] = op_t2;
    ts[9] = op_t1;
    write_reg(&mut ts, *rs, 6);
    ts
}

// ── 格式解码 ──

fn decode_format_r(ts: &[Trit; 12]) -> Result<Instruction, DecodeError> {
    let op_t2 = ts[10];
    let op_t1 = ts[9];
    let rd = extract_reg(ts, 6).ok_or(DecodeError::InvalidRegister)?;
    let rs = extract_reg(ts, 3).ok_or(DecodeError::InvalidRegister)?;
    let rt = extract_reg(ts, 0).ok_or(DecodeError::InvalidRegister)?;
    match (op_t2, op_t1) {
        (Zero, Zero) => Ok(Instruction::Nop),
        (Zero, One) => Ok(Instruction::Add { rd, rs, rt }),
        (Zero, NegOne) => Ok(Instruction::Sub { rd, rs, rt }),
        (One, Zero) => Ok(Instruction::Mul { rd, rs, rt }),
        (One, NegOne) => Ok(Instruction::Div { rd, rs, rt }),
        (One, One) => Ok(Instruction::Modulo { rd, rs, rt }),
        (NegOne, Zero) => Ok(Instruction::Mac { rd, rs, rt }),
        (NegOne, NegOne) => Ok(Instruction::Cmp { rd, rs, rt }),
        (NegOne, One) => Ok(Instruction::Sgn { rd, rs }),
    }
}

fn decode_format_i(ts: &[Trit; 12]) -> Result<Instruction, DecodeError> {
    let op_t2 = ts[10];
    let op_t1 = ts[9];
    let rd = extract_reg(ts, 6).ok_or(DecodeError::InvalidRegister)?;
    match (op_t2, op_t1) {
        // I-b: LDI
        (Zero, Zero) => {
            let imm = imm_from_trits(&ts[0..6]);
            Ok(Instruction::Ldi { rd, imm })
        }
        // I-a
        (Zero, NegOne) => {
            let rs = extract_reg(ts, 3).ok_or(DecodeError::InvalidRegister)?;
            let imm = imm_from_trits(&ts[0..3]);
            Ok(Instruction::AddI { rd, rs, imm })
        }
        (Zero, One) => {
            let rs = extract_reg(ts, 3).ok_or(DecodeError::InvalidRegister)?;
            let imm = imm_from_trits(&ts[0..3]);
            Ok(Instruction::MulI { rd, rs, imm })
        }
        (One, Zero) => {
            let rs = extract_reg(ts, 3).ok_or(DecodeError::InvalidRegister)?;
            let imm = imm_from_trits(&ts[0..3]);
            Ok(Instruction::Ld { rd, rs, imm })
        }
        (One, NegOne) => {
            let rs = extract_reg(ts, 3).ok_or(DecodeError::InvalidRegister)?;
            let imm = imm_from_trits(&ts[0..3]);
            Ok(Instruction::St { rd, rs, imm })
        }
        // 保留的 I-b 和未分配的 opcode
        (One, One) => Err(DecodeError::ReservedOpcode),
        (NegOne, Zero) | (NegOne, NegOne) | (NegOne, One) => Err(DecodeError::ReservedOpcode),
    }
}

fn decode_format_c(ts: &[Trit; 12]) -> Result<Instruction, DecodeError> {
    let op_t2 = ts[10];
    let op_t1 = ts[9];
    match (op_t2, op_t1) {
        (Zero, Zero) => {
            let offset = imm_from_trits(&ts[0..6]);
            Ok(Instruction::Jmp { offset })
        }
        (Zero, NegOne) => {
            let rs = extract_reg(ts, 6).ok_or(DecodeError::InvalidRegister)?;
            Ok(Instruction::JmpR { rs })
        }
        (Zero, One) => {
            let rs = extract_reg(ts, 6).ok_or(DecodeError::InvalidRegister)?;
            let offset = imm_from_trits(&ts[0..6]);
            Ok(Instruction::Bz { rs, offset })
        }
        (One, Zero) => {
            let rs = extract_reg(ts, 6).ok_or(DecodeError::InvalidRegister)?;
            let offset = imm_from_trits(&ts[0..6]);
            Ok(Instruction::Bn { rs, offset })
        }
        (One, NegOne) => {
            let rs = extract_reg(ts, 6).ok_or(DecodeError::InvalidRegister)?;
            let offset = imm_from_trits(&ts[0..6]);
            Ok(Instruction::Bp { rs, offset })
        }
        (One, One) => {
            let offset = imm_from_trits(&ts[0..6]);
            Ok(Instruction::Call { offset })
        }
        (NegOne, Zero) => Ok(Instruction::Ret),
        (NegOne, NegOne) => {
            let rs = extract_reg(ts, 6).ok_or(DecodeError::InvalidRegister)?;
            Ok(Instruction::Syscall { rs })
        }
        (NegOne, One) => Ok(Instruction::Halt),
    }
}

/// 从 trit slice 提取 Tryte 值（低位在前，截断或扩展至 6 trits）
fn imm_from_trits(ts: &[Trit]) -> Tryte {
    let mut extended = [Zero; 6];
    for (i, t) in ts.iter().enumerate().take(6) {
        extended[i] = *t;
    }
    Tryte::from_trits(extended)
}

#[cfg(test)]
mod tests {
    use super::*;

    const R0: Reg = Reg(0);
    const R1: Reg = Reg(1);
    const R2: Reg = Reg(2);
    const R3: Reg = Reg(3);
    const R7: Reg = Reg(7);

    fn t(v: i16) -> Tryte {
        Tryte::from_i16(v).unwrap()
    }

    // ── 寄存器编解码 ──

    #[test]
    fn test_reg_roundtrip() {
        for i in 0..8 {
            let r = Reg(i);
            let raw = r.to_raw();
            let back = Reg::from_raw(raw).unwrap();
            assert_eq!(r, back, "reg roundtrip failed for R{}", i);
        }
    }

    #[test]
    fn test_invalid_reg() {
        // 全都是 Trit 0 的组合——R0
        assert!(Reg::from_raw([Zero, Zero, Zero]).is_some());
        // 一个未使用的模式
        assert!(Reg::from_raw([One, One, One]).is_none());
        assert!(Reg::from_raw([NegOne, NegOne, NegOne]).is_none());
    }

    // ── Format R 编码/解码 ──

    #[test]
    fn test_nop_roundtrip() {
        let inst = Instruction::Nop;
        let (hi, lo) = encode(&inst);
        let decoded = decode(hi, lo).unwrap();
        assert_eq!(inst, decoded);
    }

    #[test]
    fn test_add_roundtrip() {
        let inst = Instruction::Add { rd: R0, rs: R1, rt: R2 };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_all_format_r_roundtrip() {
        let cases = [
            Instruction::Add { rd: R0, rs: R1, rt: R2 },
            Instruction::Sub { rd: R3, rs: R0, rt: R7 },
            Instruction::Mul { rd: R1, rs: R2, rt: R3 },
            Instruction::Div { rd: R7, rs: R0, rt: R1 },
            Instruction::Modulo { rd: R2, rs: R3, rt: R0 },
            Instruction::Mac { rd: R0, rs: R0, rt: R0 },
            Instruction::Cmp { rd: R1, rs: R2, rt: R3 },
            Instruction::Sgn { rd: R0, rs: R1 },
        ];
        for inst in cases {
            let (hi, lo) = encode(&inst);
            assert_eq!(decode(hi, lo).unwrap(), inst, "roundtrip failed for {:?}", inst);
        }
    }

    // ── Format I 编码/解码 ──

    #[test]
    fn test_ldi_roundtrip() {
        let inst = Instruction::Ldi { rd: R0, imm: t(42) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_ldi_negative_imm() {
        let inst = Instruction::Ldi { rd: R1, imm: t(-364) };
        let (hi, lo) = encode(&inst);
        let decoded = decode(hi, lo).unwrap();
        if let Instruction::Ldi { rd, imm } = decoded {
            assert_eq!(rd, R1);
            assert_eq!(imm.to_i16(), -364);
        } else {
            panic!("wrong instruction type");
        }
    }

    #[test]
    fn test_addi_roundtrip() {
        let inst = Instruction::AddI { rd: R0, rs: R1, imm: t(3) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_muli_roundtrip() {
        let inst = Instruction::MulI { rd: R2, rs: R3, imm: t(-5) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_ld_roundtrip() {
        let inst = Instruction::Ld { rd: R0, rs: R7, imm: t(-13) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_st_roundtrip() {
        let inst = Instruction::St { rd: R3, rs: R0, imm: t(7) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    // ── Format C 编码/解码 ──

    #[test]
    fn test_jmp_roundtrip() {
        let inst = Instruction::Jmp { offset: t(100) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_jmpr_roundtrip() {
        let inst = Instruction::JmpR { rs: R0 };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_bz_roundtrip() {
        let inst = Instruction::Bz { rs: R2, offset: t(-50) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_bn_roundtrip() {
        let inst = Instruction::Bn { rs: R0, offset: t(200) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_bp_roundtrip() {
        let inst = Instruction::Bp { rs: R7, offset: Tryte::MAX };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_call_roundtrip() {
        let inst = Instruction::Call { offset: t(-10) };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_ret_roundtrip() {
        let inst = Instruction::Ret;
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_syscall_roundtrip() {
        let inst = Instruction::Syscall { rs: R1 };
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    #[test]
    fn test_halt_roundtrip() {
        let inst = Instruction::Halt;
        let (hi, lo) = encode(&inst);
        assert_eq!(decode(hi, lo).unwrap(), inst);
    }

    // ── 全覆盖 ──

    #[test]
    fn test_every_instruction_roundtrips() {
        let all = vec![
            Instruction::Nop,
            Instruction::Add { rd: R0, rs: R1, rt: R2 },
            Instruction::Sub { rd: R3, rs: R0, rt: R7 },
            Instruction::Mul { rd: R1, rs: R2, rt: R3 },
            Instruction::Div { rd: R7, rs: R0, rt: R1 },
            Instruction::Modulo { rd: R2, rs: R3, rt: R0 },
            Instruction::Mac { rd: R0, rs: R0, rt: R0 },
            Instruction::Cmp { rd: R1, rs: R2, rt: R3 },
            Instruction::Sgn { rd: R0, rs: R1 },
            Instruction::AddI { rd: R0, rs: R1, imm: t(-13) },
            Instruction::MulI { rd: R2, rs: R3, imm: t(12) },
            Instruction::Ld { rd: R0, rs: R7, imm: t(0) },
            Instruction::St { rd: R3, rs: R0, imm: t(7) },
            Instruction::Ldi { rd: R0, imm: t(364) },
            Instruction::Jmp { offset: t(1) },
            Instruction::JmpR { rs: R0 },
            Instruction::Bz { rs: R2, offset: t(-1) },
            Instruction::Bn { rs: R0, offset: t(0) },
            Instruction::Bp { rs: R7, offset: t(364) },
            Instruction::Call { offset: t(-364) },
            Instruction::Ret,
            Instruction::Syscall { rs: R3 },
            Instruction::Halt,
        ];
        for inst in all {
            let (hi, lo) = encode(&inst);
            let decoded = decode(hi, lo).unwrap();
            assert_eq!(inst, decoded, "roundtrip failed for {:?}", inst);
        }
    }
}
