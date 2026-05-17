use crate::instruction::{decode, Instruction, Reg};
use crate::tryte::{Tryte, TryteError};

pub const MEM_SIZE: usize = 59049;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    Halted,
    BadAddress,
    BadInstruction,
}

pub struct VM {
    pub regs: [Tryte; 8],
    pub pc: i32,
    pub sp: i32,
    pub memory: Vec<Tryte>,
    pub running: bool,
    pub output: String,
    pub exit_code: i32,
}

impl VM {
    pub fn new() -> Self {
        VM {
            regs: [Tryte::from_i16(0).unwrap(); 8],
            pc: 0,
            sp: (MEM_SIZE - 1) as i32,
            memory: vec![Tryte::from_i16(0).unwrap(); MEM_SIZE],
            running: true,
            output: String::new(),
            exit_code: 0,
        }
    }

    pub fn mem_read(&self, addr: i32) -> Result<Tryte, ExecError> {
        if addr < 0 || addr >= MEM_SIZE as i32 {
            return Err(ExecError::BadAddress);
        }
        Ok(self.memory[addr as usize])
    }

    pub fn mem_write(&mut self, addr: i32, val: Tryte) -> Result<(), ExecError> {
        if addr < 0 || addr >= MEM_SIZE as i32 {
            return Err(ExecError::BadAddress);
        }
        self.memory[addr as usize] = val;
        Ok(())
    }

    pub fn exec_inst(&mut self, inst: &Instruction) -> Result<(), ExecError> {
        match *inst {
            Instruction::Nop => {}

            Instruction::Add { rd, rs, rt } => {
                self.exec_binary_op(rd, rs, rt, |a, b| a.add(b));
            }
            Instruction::Sub { rd, rs, rt } => {
                self.exec_binary_op(rd, rs, rt, |a, b| a.sub(b));
            }
            Instruction::Mul { rd, rs, rt } => {
                let a = self.regs[rs.idx() as usize];
                let b = self.regs[rt.idx() as usize];
                // MUL returns (hi, lo) — we only use lo, check for overflow
                let (_hi, lo) = a.mul_wide(b);
                if _hi.to_i16() != 0 {
                    self.regs[rd.idx() as usize] = if a.to_i16() >= 0 && b.to_i16() >= 0
                        || a.to_i16() < 0 && b.to_i16() < 0
                    {
                        Tryte::MAX
                    } else {
                        Tryte::MIN
                    };
                    self.regs[7] = Tryte::from_i16(if _hi.to_i16() > 0 { 1 } else { -1 }).unwrap();
                } else {
                    self.regs[rd.idx() as usize] = lo;
                    self.regs[7] = Tryte::from_i16(0).unwrap();
                }
            }
            Instruction::Div { rd, rs, rt } => {
                match self.regs[rs.idx() as usize].div(self.regs[rt.idx() as usize]) {
                    Ok(v) => {
                        self.regs[rd.idx() as usize] = v;
                        self.regs[7] = Tryte::ZERO;
                    }
                    Err(TryteError::DivisionByZero) => {
                        self.regs[rd.idx() as usize] = Tryte::ZERO;
                        self.regs[7] = Tryte::NEG_ONE;
                    }
                    Err(TryteError::Overflow(_)) => unreachable!(),
                }
            }
            Instruction::Modulo { rd, rs, rt } => {
                match self.regs[rs.idx() as usize].rem(self.regs[rt.idx() as usize]) {
                    Ok(v) => {
                        self.regs[rd.idx() as usize] = v;
                        self.regs[7] = Tryte::ZERO;
                    }
                    Err(TryteError::DivisionByZero) => {
                        self.regs[rd.idx() as usize] = Tryte::ZERO;
                        self.regs[7] = Tryte::NEG_ONE;
                    }
                    Err(TryteError::Overflow(_)) => unreachable!(),
                }
            }
            Instruction::Mac { rd, rs, rt } => {
                let a = self.regs[rs.idx() as usize];
                let b = self.regs[rt.idx() as usize];
                let (_hi, prod) = a.mul_wide(b);
                let acc = self.regs[rd.idx() as usize];
                match acc.add(prod) {
                    Ok(v) => {
                        self.regs[rd.idx() as usize] = v;
                        self.regs[7] = Tryte::ZERO;
                    }
                    Err(TryteError::Overflow(v)) => {
                        self.regs[rd.idx() as usize] = v;
                        self.regs[7] = Tryte::from_i16(if v.to_i16() > 0 { 1 } else { -1 }).unwrap();
                    }
                    Err(TryteError::DivisionByZero) => unreachable!(),
                }
            }
            Instruction::Cmp { rd, rs, rt } => {
                let a = self.regs[rs.idx() as usize];
                let b = self.regs[rt.idx() as usize];
                let diff = a.to_i16() - b.to_i16();
                self.regs[rd.idx() as usize] = if diff > 0 {
                    Tryte::ONE
                } else if diff < 0 {
                    Tryte::NEG_ONE
                } else {
                    Tryte::ZERO
                };
            }
            Instruction::Sgn { rd, rs } => {
                let v = self.regs[rs.idx() as usize].to_i16();
                self.regs[rd.idx() as usize] = if v > 0 {
                    Tryte::ONE
                } else if v < 0 {
                    Tryte::NEG_ONE
                } else {
                    Tryte::ZERO
                };
            }

            // ── Format I-a ──

            Instruction::AddI { rd, rs, imm } => {
                let a = self.regs[rs.idx() as usize];
                match a.add(imm) {
                    Ok(v) => {
                        self.regs[rd.idx() as usize] = v;
                        self.regs[7] = Tryte::ZERO;
                    }
                    Err(TryteError::Overflow(v)) => {
                        self.regs[rd.idx() as usize] = v;
                        self.regs[7] = Tryte::from_i16(if v.to_i16() > 0 { 1 } else { -1 }).unwrap();
                    }
                    Err(TryteError::DivisionByZero) => unreachable!(),
                }
            }
            Instruction::MulI { rd, rs, imm } => {
                let a = self.regs[rs.idx() as usize];
                let b = imm;
                let (_hi, lo) = a.mul_wide(b);
                if _hi.to_i16() != 0 {
                    self.regs[rd.idx() as usize] = if a.to_i16() >= 0 && b.to_i16() >= 0
                        || a.to_i16() < 0 && b.to_i16() < 0
                    {
                        Tryte::MAX
                    } else {
                        Tryte::MIN
                    };
                    self.regs[7] = Tryte::from_i16(if _hi.to_i16() > 0 { 1 } else { -1 }).unwrap();
                } else {
                    self.regs[rd.idx() as usize] = lo;
                    self.regs[7] = Tryte::ZERO;
                }
            }
            Instruction::Ld { rd, rs, imm } => {
                let addr = match self.regs[rs.idx() as usize].add(imm) {
                    Ok(v) => {
                        self.regs[7] = Tryte::ZERO;
                        v
                    }
                    Err(TryteError::Overflow(v)) => {
                        self.regs[7] = Tryte::from_i16(if v.to_i16() > 0 { 1 } else { -1 }).unwrap();
                        v
                    }
                    Err(TryteError::DivisionByZero) => unreachable!(),
                };
                self.regs[rd.idx() as usize] = self.mem_read(addr.to_i16() as i32)?;
            }
            Instruction::St { rd, rs, imm } => {
                let addr = match self.regs[rs.idx() as usize].add(imm) {
                    Ok(v) => {
                        self.regs[7] = Tryte::ZERO;
                        v
                    }
                    Err(TryteError::Overflow(v)) => {
                        self.regs[7] = Tryte::from_i16(if v.to_i16() > 0 { 1 } else { -1 }).unwrap();
                        v
                    }
                    Err(TryteError::DivisionByZero) => unreachable!(),
                };
                self.mem_write(addr.to_i16() as i32, self.regs[rd.idx() as usize])?;
            }

            // ── Format I-b ──

            Instruction::Ldi { rd, imm } => {
                self.regs[rd.idx() as usize] = imm;
            }

            _ => {}
        }
        Ok(())
    }

    fn exec_binary_op(
        &mut self, rd: Reg, rs: Reg, rt: Reg,
        op: fn(Tryte, Tryte) -> Result<Tryte, TryteError>,
    ) {
        let a = self.regs[rs.idx() as usize];
        let b = self.regs[rt.idx() as usize];
        match op(a, b) {
            Ok(v) => {
                self.regs[rd.idx() as usize] = v;
                self.regs[7] = Tryte::ZERO;
            }
            Err(TryteError::Overflow(v)) => {
                self.regs[rd.idx() as usize] = v;
                self.regs[7] = Tryte::from_i16(if v.to_i16() > 0 { 1 } else { -1 }).unwrap();
            }
            Err(TryteError::DivisionByZero) => unreachable!(),
        }
    }

    fn stack_push(&mut self, addr: i32) -> Result<(), ExecError> {
        if self.sp < 2 {
            return Err(ExecError::BadAddress);
        }
        let lo_val = addr % 729;
        let hi_val = addr / 729;
        self.sp -= 1;
        self.memory[self.sp as usize] = Tryte::from_i16(lo_val as i16 - 364).unwrap();
        self.sp -= 1;
        self.memory[self.sp as usize] = Tryte::from_i16(hi_val as i16 - 364).unwrap();
        Ok(())
    }

    fn stack_pop(&mut self) -> Result<i32, ExecError> {
        if self.sp as i32 >= (MEM_SIZE - 1) as i32 {
            return Err(ExecError::BadAddress);
        }
        let hi = self.memory[self.sp as usize].to_i16() as i32 + 364;
        self.sp += 1;
        let lo = self.memory[self.sp as usize].to_i16() as i32 + 364;
        self.sp += 1;
        Ok(hi * 729 + lo)
    }

    pub fn step(&mut self) -> Result<(), ExecError> {
        if !self.running {
            return Err(ExecError::Halted);
        }
        let hi = self.mem_read(self.pc)?;
        let lo = self.mem_read(self.pc + 1)?;
        let inst = decode(hi, lo).map_err(|_| ExecError::BadInstruction)?;

        match inst {
            Instruction::Jmp { offset } => {
                self.pc += offset.to_i16() as i32;
            }
            Instruction::JmpR { rs } => {
                self.pc = self.regs[rs.idx() as usize].to_i16() as i32;
            }
            Instruction::Bz { rs, offset } => {
                if self.regs[rs.idx() as usize].to_i16() == 0 {
                    self.pc += offset.to_i16() as i32;
                } else {
                    self.pc += 2;
                }
            }
            Instruction::Bn { rs, offset } => {
                if self.regs[rs.idx() as usize].to_i16() < 0 {
                    self.pc += offset.to_i16() as i32;
                } else {
                    self.pc += 2;
                }
            }
            Instruction::Bp { rs, offset } => {
                if self.regs[rs.idx() as usize].to_i16() > 0 {
                    self.pc += offset.to_i16() as i32;
                } else {
                    self.pc += 2;
                }
            }
            Instruction::Call { offset } => {
                let ret = self.pc + 2;
                self.stack_push(ret)?;
                self.pc += offset.to_i16() as i32;
            }
            Instruction::Ret => {
                self.pc = self.stack_pop()?;
            }
            Instruction::Syscall { rs } => {
                let func = self.regs[rs.idx() as usize].to_i16();
                match func {
                    1 => { // EXIT
                        self.exit_code = self.regs[0].to_i16() as i32;
                        self.running = false;
                    }
                    2 => { // PRINT_T
                        self.output.push_str(&self.regs[0].to_string());
                        self.pc += 2;
                    }
                    3 => { // PRINT_D
                        self.output.push_str(&self.regs[0].to_i16().to_string());
                        self.pc += 2;
                    }
                    4 => { // PRINT_C
                        if let Some(c) = char::from_u32(self.regs[0].to_i16() as u32) {
                            self.output.push(c);
                        }
                        self.pc += 2;
                    }
                    7 => { // PRINT_S
                        let mut addr = self.regs[0].to_i16() as i32;
                        let start = addr;
                        loop {
                            if addr - start >= 256 { break; }
                            let val = self.mem_read(addr)?;
                            let byte = val.to_i16();
                            if byte == 0 { break; }
                            if let Some(c) = char::from_u32(byte as u32) {
                                self.output.push(c);
                            }
                            addr += 1;
                        }
                        self.pc += 2;
                    }
                    _ => {
                        self.pc += 2; // invalid func — continue
                    }
                }
            }
            Instruction::Halt => {
                self.running = false;
            }
            _ => {
                self.exec_inst(&inst)?;
                self.pc += 2;
            }
        }
        Ok(())
    }

    pub fn run(&mut self) {
        while self.running {
            if let Err(_) = self.step() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::encode;

    fn r(i: u8) -> Reg { Reg(i) }
    fn t(v: i16) -> Tryte { Tryte::from_i16(v).unwrap() }

    // ── VM-01~04: 初始状态 ──

    #[test]
    fn test_vm_initial_regs() {
        let vm = VM::new();
        for i in 0..8 {
            assert_eq!(vm.regs[i].to_i16(), 0, "R{} should be 0", i);
        }
    }

    #[test]
    fn test_vm_initial_pc() {
        let vm = VM::new();
        assert_eq!(vm.pc, 0);
    }

    #[test]
    fn test_vm_initial_sp() {
        let vm = VM::new();
        assert_eq!(vm.sp, (MEM_SIZE - 1) as i32);
    }

    #[test]
    fn test_vm_initial_memory() {
        let vm = VM::new();
        assert_eq!(vm.memory.len(), MEM_SIZE);
        for i in 0..MEM_SIZE {
            assert_eq!(vm.memory[i].to_i16(), 0, "mem[{}] should be 0", i);
        }
    }

    #[test]
    fn test_vm_initial_running() {
        let vm = VM::new();
        assert!(vm.running);
    }

    // ── VM-05~07: 内存读写 ──

    #[test]
    fn test_mem_write_then_read() {
        let mut vm = VM::new();
        let val = Tryte::from_i16(42).unwrap();
        assert!(vm.mem_write(100, val).is_ok());
        assert_eq!(vm.mem_read(100).unwrap(), val);
    }

    #[test]
    fn test_mem_write_addr_zero() {
        let mut vm = VM::new();
        let val = Tryte::from_i16(-364).unwrap();
        assert!(vm.mem_write(0, val).is_ok());
        assert_eq!(vm.mem_read(0).unwrap(), val);
    }

    #[test]
    fn test_mem_write_max_addr() {
        let mut vm = VM::new();
        let val = Tryte::from_i16(364).unwrap();
        assert!(vm.mem_write((MEM_SIZE - 1) as i32, val).is_ok());
        assert_eq!(vm.mem_read((MEM_SIZE - 1) as i32).unwrap(), val);
    }

    // ── VM-08~09: 越界 ──

    #[test]
    fn test_mem_read_negative_addr() {
        let vm = VM::new();
        assert_eq!(vm.mem_read(-1), Err(ExecError::BadAddress));
    }

    #[test]
    fn test_mem_read_oob() {
        let vm = VM::new();
        assert_eq!(vm.mem_read(MEM_SIZE as i32), Err(ExecError::BadAddress));
    }

    #[test]
    fn test_mem_write_negative_addr() {
        let mut vm = VM::new();
        let val = Tryte::from_i16(1).unwrap();
        assert_eq!(vm.mem_write(-1, val), Err(ExecError::BadAddress));
    }

    #[test]
    fn test_mem_write_oob() {
        let mut vm = VM::new();
        let val = Tryte::from_i16(1).unwrap();
        assert_eq!(vm.mem_write(MEM_SIZE as i32, val), Err(ExecError::BadAddress));
    }

    // ── R-01~07: ADD ──

    #[test]
    fn test_add_normal() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.regs[2] = t(20);
        vm.exec_inst(&Instruction::Add { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(30));
        assert_eq!(vm.regs[7], t(0));
    }

    #[test]
    fn test_add_neg_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(-10);
        vm.regs[2] = t(-20);
        vm.exec_inst(&Instruction::Add { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(-30));
    }

    #[test]
    fn test_add_cancel() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.regs[2] = t(-10);
        vm.exec_inst(&Instruction::Add { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
    }

    #[test]
    fn test_add_overflow_pos() {
        let mut vm = VM::new();
        vm.regs[1] = t(364);
        vm.regs[2] = t(1);
        vm.exec_inst(&Instruction::Add { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(364)); // saturated
        assert_eq!(vm.regs[7], t(1));   // positive overflow
    }

    #[test]
    fn test_add_overflow_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(-364);
        vm.regs[2] = t(-1);
        vm.exec_inst(&Instruction::Add { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(-364));
        assert_eq!(vm.regs[7], t(-1));
    }

    #[test]
    fn test_add_no_overflow_at_boundary() {
        let mut vm = VM::new();
        vm.regs[1] = t(200);
        vm.regs[2] = t(164);
        vm.exec_inst(&Instruction::Add { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(364));
        assert_eq!(vm.regs[7], t(0));
    }

    // ── R-10~14: SUB ──

    #[test]
    fn test_sub_normal() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.regs[2] = t(3);
        vm.exec_inst(&Instruction::Sub { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(7));
    }

    #[test]
    fn test_sub_neg_rhs() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.regs[2] = t(-3);
        vm.exec_inst(&Instruction::Sub { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(13));
    }

    #[test]
    fn test_sub_overflow_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(-364);
        vm.regs[2] = t(1);
        vm.exec_inst(&Instruction::Sub { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(-364));
        assert_eq!(vm.regs[7], t(-1));
    }

    #[test]
    fn test_sub_overflow_pos() {
        let mut vm = VM::new();
        vm.regs[1] = t(364);
        vm.regs[2] = t(-1);
        vm.exec_inst(&Instruction::Sub { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(364));
    }

    // ── R-20~27: MUL ──

    #[test]
    fn test_mul_normal() {
        let mut vm = VM::new();
        vm.regs[1] = t(3);
        vm.regs[2] = t(5);
        vm.exec_inst(&Instruction::Mul { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(15));
    }

    #[test]
    fn test_mul_neg_pos() {
        let mut vm = VM::new();
        vm.regs[1] = t(-3);
        vm.regs[2] = t(5);
        vm.exec_inst(&Instruction::Mul { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(-15));
    }

    #[test]
    fn test_mul_neg_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(-3);
        vm.regs[2] = t(-5);
        vm.exec_inst(&Instruction::Mul { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(15));
    }

    #[test]
    fn test_mul_by_zero() {
        let mut vm = VM::new();
        vm.regs[1] = t(364);
        vm.regs[2] = t(0);
        vm.exec_inst(&Instruction::Mul { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
    }

    #[test]
    fn test_mul_overflow_saturate() {
        let mut vm = VM::new();
        vm.regs[1] = t(20);
        vm.regs[2] = t(20);
        vm.exec_inst(&Instruction::Mul { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(364));
        assert_eq!(vm.regs[7], t(1));
    }

    // ── R-30~37: DIV ──

    #[test]
    fn test_div_normal() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.regs[2] = t(3);
        vm.exec_inst(&Instruction::Div { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(3));
    }

    #[test]
    fn test_div_neg_pos() {
        let mut vm = VM::new();
        vm.regs[1] = t(-10);
        vm.regs[2] = t(3);
        vm.exec_inst(&Instruction::Div { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(-3));
    }

    #[test]
    fn test_div_by_zero() {
        let mut vm = VM::new();
        vm.regs[1] = t(5);
        vm.regs[2] = t(0);
        vm.exec_inst(&Instruction::Div { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
        assert_eq!(vm.regs[7], t(-1));
    }

    // ── R-40~45: MOD ──

    #[test]
    fn test_mod_normal() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.regs[2] = t(3);
        vm.exec_inst(&Instruction::Modulo { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(1));
    }

    #[test]
    fn test_mod_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(-10);
        vm.regs[2] = t(3);
        vm.exec_inst(&Instruction::Modulo { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(-1));
    }

    #[test]
    fn test_mod_by_zero() {
        let mut vm = VM::new();
        vm.regs[1] = t(5);
        vm.regs[2] = t(0);
        vm.exec_inst(&Instruction::Modulo { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
        assert_eq!(vm.regs[7], t(-1));
    }

    // ── R-50~54: MAC ──

    #[test]
    fn test_mac_once() {
        let mut vm = VM::new();
        vm.regs[0] = t(0);
        vm.regs[1] = t(3);
        vm.regs[2] = t(5);
        vm.exec_inst(&Instruction::Mac { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(15));
    }

    #[test]
    fn test_mac_multiple() {
        let mut vm = VM::new();
        vm.regs[0] = t(0);
        // MAC with (3,5) then (-1,2) then (1,4)
        vm.regs[1] = t(3); vm.regs[2] = t(5);
        vm.exec_inst(&Instruction::Mac { rd: r(0), rs: r(1), rt: r(2) }).unwrap(); // 15
        vm.regs[1] = t(-1); vm.regs[2] = t(2);
        vm.exec_inst(&Instruction::Mac { rd: r(0), rs: r(1), rt: r(2) }).unwrap(); // 13
        vm.regs[1] = t(1); vm.regs[2] = t(4);
        vm.exec_inst(&Instruction::Mac { rd: r(0), rs: r(1), rt: r(2) }).unwrap(); // 17
        assert_eq!(vm.regs[0], t(17));
    }

    // ── R-60~65: CMP ──

    #[test]
    fn test_cmp_greater() {
        let mut vm = VM::new();
        vm.regs[1] = t(5);
        vm.regs[2] = t(3);
        vm.exec_inst(&Instruction::Cmp { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(1));
    }

    #[test]
    fn test_cmp_less() {
        let mut vm = VM::new();
        vm.regs[1] = t(3);
        vm.regs[2] = t(5);
        vm.exec_inst(&Instruction::Cmp { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(-1));
    }

    #[test]
    fn test_cmp_equal() {
        let mut vm = VM::new();
        vm.regs[1] = t(-7);
        vm.regs[2] = t(-7);
        vm.exec_inst(&Instruction::Cmp { rd: r(0), rs: r(1), rt: r(2) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
    }

    // ── R-70~74: SGN ──

    #[test]
    fn test_sgn_pos() {
        let mut vm = VM::new();
        vm.regs[1] = t(42);
        vm.exec_inst(&Instruction::Sgn { rd: r(0), rs: r(1) }).unwrap();
        assert_eq!(vm.regs[0], t(1));
    }

    #[test]
    fn test_sgn_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(-42);
        vm.exec_inst(&Instruction::Sgn { rd: r(0), rs: r(1) }).unwrap();
        assert_eq!(vm.regs[0], t(-1));
    }

    #[test]
    fn test_sgn_zero() {
        let mut vm = VM::new();
        vm.regs[1] = t(0);
        vm.exec_inst(&Instruction::Sgn { rd: r(0), rs: r(1) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
    }

    // ── I-01~04: LDI ──

    #[test]
    fn test_ldi_pos() {
        let mut vm = VM::new();
        vm.exec_inst(&Instruction::Ldi { rd: r(0), imm: t(42) }).unwrap();
        assert_eq!(vm.regs[0], t(42));
    }

    #[test]
    fn test_ldi_neg() {
        let mut vm = VM::new();
        vm.exec_inst(&Instruction::Ldi { rd: r(0), imm: t(-13) }).unwrap();
        assert_eq!(vm.regs[0], t(-13));
    }

    #[test]
    fn test_ldi_zero() {
        let mut vm = VM::new();
        vm.exec_inst(&Instruction::Ldi { rd: r(0), imm: t(0) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
    }

    #[test]
    fn test_ldi_multiple_regs() {
        let mut vm = VM::new();
        vm.exec_inst(&Instruction::Ldi { rd: r(1), imm: t(10) }).unwrap();
        vm.exec_inst(&Instruction::Ldi { rd: r(2), imm: t(20) }).unwrap();
        assert_eq!(vm.regs[1], t(10));
        assert_eq!(vm.regs[2], t(20));
    }

    // ── I-10~13: ADDI ──

    #[test]
    fn test_addi_pos_imm() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.exec_inst(&Instruction::AddI { rd: r(0), rs: r(1), imm: t(5) }).unwrap();
        assert_eq!(vm.regs[0], t(15));
        assert_eq!(vm.regs[7], t(0));
    }

    #[test]
    fn test_addi_neg_imm() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.exec_inst(&Instruction::AddI { rd: r(0), rs: r(1), imm: t(-3) }).unwrap();
        assert_eq!(vm.regs[0], t(7));
    }

    #[test]
    fn test_addi_overflow() {
        let mut vm = VM::new();
        vm.regs[1] = t(364);
        vm.exec_inst(&Instruction::AddI { rd: r(0), rs: r(1), imm: t(13) }).unwrap();
        assert_eq!(vm.regs[0], t(364));
        assert_eq!(vm.regs[7], t(1));
    }

    #[test]
    fn test_addi_overflow_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(-364);
        vm.exec_inst(&Instruction::AddI { rd: r(0), rs: r(1), imm: t(-13) }).unwrap();
        assert_eq!(vm.regs[0], t(-364));
        assert_eq!(vm.regs[7], t(-1));
    }

    // ── I-20~23: MULI ──

    #[test]
    fn test_muli_pos() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.exec_inst(&Instruction::MulI { rd: r(0), rs: r(1), imm: t(3) }).unwrap();
        assert_eq!(vm.regs[0], t(30));
    }

    #[test]
    fn test_muli_neg() {
        let mut vm = VM::new();
        vm.regs[1] = t(10);
        vm.exec_inst(&Instruction::MulI { rd: r(0), rs: r(1), imm: t(-3) }).unwrap();
        assert_eq!(vm.regs[0], t(-30));
    }

    #[test]
    fn test_muli_by_zero() {
        let mut vm = VM::new();
        vm.regs[1] = t(42);
        vm.exec_inst(&Instruction::MulI { rd: r(0), rs: r(1), imm: t(0) }).unwrap();
        assert_eq!(vm.regs[0], t(0));
    }

    #[test]
    fn test_muli_overflow() {
        let mut vm = VM::new();
        vm.regs[1] = t(30);
        vm.exec_inst(&Instruction::MulI { rd: r(0), rs: r(1), imm: t(13) }).unwrap();
        assert_eq!(vm.regs[0], t(364));
        assert_eq!(vm.regs[7], t(1));
    }

    // ── I-30~33: LD ──

    fn setup_memory(vm: &mut VM) {
        // write some test values at low addresses
        vm.memory[10] = t(42);
        vm.memory[20] = t(-7);
        vm.memory[0] = t(99);
    }

    #[test]
    fn test_ld_from_mem() {
        let mut vm = VM::new();
        setup_memory(&mut vm);
        vm.regs[1] = t(10);
        vm.exec_inst(&Instruction::Ld { rd: r(0), rs: r(1), imm: t(0) }).unwrap();
        assert_eq!(vm.regs[0], t(42));
    }

    #[test]
    fn test_ld_with_offset() {
        let mut vm = VM::new();
        setup_memory(&mut vm);
        vm.regs[1] = t(15);
        vm.exec_inst(&Instruction::Ld { rd: r(0), rs: r(1), imm: t(5) }).unwrap();
        assert_eq!(vm.regs[0], t(-7)); // mem[20]
    }

    #[test]
    fn test_ld_neg_offset() {
        let mut vm = VM::new();
        setup_memory(&mut vm);
        vm.regs[1] = t(12);
        vm.exec_inst(&Instruction::Ld { rd: r(0), rs: r(1), imm: t(-2) }).unwrap();
        assert_eq!(vm.regs[0], t(42)); // mem[10]
    }

    #[test]
    fn test_ld_addr_zero() {
        let mut vm = VM::new();
        setup_memory(&mut vm);
        vm.regs[1] = t(0);
        vm.exec_inst(&Instruction::Ld { rd: r(0), rs: r(1), imm: t(0) }).unwrap();
        assert_eq!(vm.regs[0], t(99));
    }

    #[test]
    fn test_ld_oob() {
        let mut vm = VM::new();
        vm.regs[1] = t(0);
        let result = vm.exec_inst(&Instruction::Ld { rd: r(0), rs: r(1), imm: t(-1) });
        assert_eq!(result, Err(ExecError::BadAddress));
    }

    // ── I-40~43: ST ──

    #[test]
    fn test_st_to_mem() {
        let mut vm = VM::new();
        vm.regs[0] = t(42);
        vm.regs[1] = t(10);
        vm.exec_inst(&Instruction::St { rd: r(0), rs: r(1), imm: t(0) }).unwrap();
        assert_eq!(vm.memory[10], t(42));
    }

    #[test]
    fn test_st_with_offset() {
        let mut vm = VM::new();
        vm.regs[0] = t(77);
        vm.regs[1] = t(20);
        vm.exec_inst(&Instruction::St { rd: r(0), rs: r(1), imm: t(5) }).unwrap();
        assert_eq!(vm.memory[25], t(77));
    }

    #[test]
    fn test_st_neg_offset() {
        let mut vm = VM::new();
        vm.regs[0] = t(-1);
        vm.regs[1] = t(15);
        vm.exec_inst(&Instruction::St { rd: r(0), rs: r(1), imm: t(-3) }).unwrap();
        assert_eq!(vm.memory[12], t(-1));
    }

    #[test]
    fn test_st_oob() {
        let mut vm = VM::new();
        vm.regs[0] = t(5);
        vm.regs[1] = t(0);
        let result = vm.exec_inst(&Instruction::St { rd: r(0), rs: r(1), imm: t(-1) });
        assert_eq!(result, Err(ExecError::BadAddress));
    }

    // ── C-01~04: JMP ──

    fn place_inst(vm: &mut VM, addr: i32, inst: &Instruction) {
        let (hi, lo) = encode(inst);
        vm.memory[addr as usize] = hi;
        vm.memory[(addr + 1) as usize] = lo;
    }

    #[test]
    fn test_jmp_forward() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Jmp { offset: t(4) });
        place_inst(&mut vm, 2, &Instruction::Nop); // skipped
        place_inst(&mut vm, 4, &Instruction::Nop); // land here
        vm.step().unwrap();
        assert_eq!(vm.pc, 4);
    }

    #[test]
    fn test_jmp_backward() {
        let mut vm = VM::new();
        place_inst(&mut vm, 10, &Instruction::Jmp { offset: t(-6) });
        place_inst(&mut vm, 4, &Instruction::Nop); // land here
        place_inst(&mut vm, 6, &Instruction::Nop); // skipped
        place_inst(&mut vm, 8, &Instruction::Nop); // skipped
        vm.pc = 10;
        vm.step().unwrap();
        assert_eq!(vm.pc, 4);
    }

    #[test]
    fn test_jmp_zero_offset_loop() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Jmp { offset: t(0) });
        vm.step().unwrap();
        assert_eq!(vm.pc, 0); // infinite loop
    }

    #[test]
    fn test_jmp_chain() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Jmp { offset: t(4) });
        place_inst(&mut vm, 4, &Instruction::Jmp { offset: t(4) });
        place_inst(&mut vm, 8, &Instruction::Nop);
        vm.step().unwrap();
        assert_eq!(vm.pc, 4);
        vm.step().unwrap();
        assert_eq!(vm.pc, 8);
    }

    // ── C-10~12: JMPR ──

    #[test]
    fn test_jmpr() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::JmpR { rs: r(1) });
        place_inst(&mut vm, 10, &Instruction::Nop); // land here (R1=10)
        vm.regs[1] = t(10);
        vm.step().unwrap();
        assert_eq!(vm.pc, 10);
    }

    #[test]
    fn test_jmpr_zero() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::JmpR { rs: r(1) });
        vm.regs[1] = t(0);
        vm.step().unwrap();
        assert_eq!(vm.pc, 0);
    }

    // ── C-20~24: BZ ──

    #[test]
    fn test_bz_taken() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Bz { rs: r(1), offset: t(4) });
        place_inst(&mut vm, 4, &Instruction::Nop); // land here
        vm.regs[1] = t(0);
        vm.step().unwrap();
        assert_eq!(vm.pc, 4);
    }

    #[test]
    fn test_bz_not_taken() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Bz { rs: r(1), offset: t(4) });
        place_inst(&mut vm, 2, &Instruction::Nop);
        vm.regs[1] = t(1);
        vm.step().unwrap();
        assert_eq!(vm.pc, 2);
    }

    // ── C-30~34: BN ──

    #[test]
    fn test_bn_taken() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Bn { rs: r(1), offset: t(4) });
        place_inst(&mut vm, 4, &Instruction::Nop);
        vm.regs[1] = t(-1);
        vm.step().unwrap();
        assert_eq!(vm.pc, 4);
    }

    #[test]
    fn test_bn_not_taken() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Bn { rs: r(1), offset: t(4) });
        place_inst(&mut vm, 2, &Instruction::Nop);
        vm.regs[1] = t(0);
        vm.step().unwrap();
        assert_eq!(vm.pc, 2);
    }

    // ── C-40~44: BP ──

    #[test]
    fn test_bp_taken() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Bp { rs: r(1), offset: t(4) });
        place_inst(&mut vm, 4, &Instruction::Nop);
        vm.regs[1] = t(1);
        vm.step().unwrap();
        assert_eq!(vm.pc, 4);
    }

    #[test]
    fn test_bp_not_taken() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Bp { rs: r(1), offset: t(4) });
        place_inst(&mut vm, 2, &Instruction::Nop);
        vm.regs[1] = t(0);
        vm.step().unwrap();
        assert_eq!(vm.pc, 2);
    }

    // ── C-50~53: CALL/RET ──

    #[test]
    fn test_call_and_ret() {
        let mut vm = VM::new();
        // PC=0: CALL +4 → push 2 (ret addr), jump to 4
        // PC=4: RET → pop ret addr, jump to 2
        // PC=2: NOP
        place_inst(&mut vm, 0, &Instruction::Call { offset: t(4) });
        place_inst(&mut vm, 4, &Instruction::Ret);
        place_inst(&mut vm, 2, &Instruction::Nop);
        vm.step().unwrap();
        // After CALL: PC = 4, stack has [ret=2]
        assert_eq!(vm.pc, 4);
        vm.step().unwrap();
        // After RET: PC = 2
        assert_eq!(vm.pc, 2);
    }

    #[test]
    fn test_nested_call() {
        let mut vm = VM::new();
        // PC=0: CALL +4 → push 2, jump to 4
        // PC=4: CALL +4 → push 6, jump to 8
        // PC=8: RET → pop 6
        // PC=6: RET → pop 2
        // PC=2: NOP
        place_inst(&mut vm, 0, &Instruction::Call { offset: t(4) });
        place_inst(&mut vm, 4, &Instruction::Call { offset: t(4) });
        place_inst(&mut vm, 8, &Instruction::Ret);
        place_inst(&mut vm, 6, &Instruction::Ret);
        place_inst(&mut vm, 2, &Instruction::Nop);
        vm.step().unwrap(); assert_eq!(vm.pc, 4);
        vm.step().unwrap(); assert_eq!(vm.pc, 8);  // inner call
        vm.step().unwrap(); assert_eq!(vm.pc, 6);  // inner ret
        vm.step().unwrap(); assert_eq!(vm.pc, 2);  // outer ret
    }

    // ── C-60~62: SYSCALL ──

    #[test]
    fn test_syscall_exit() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(1);  // EXIT
        vm.regs[0] = t(42); // exit code
        vm.step().unwrap();
        assert_eq!(vm.exit_code, 42);
        assert!(!vm.running);
    }

    #[test]
    fn test_syscall_exit_neg() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(1);  // EXIT
        vm.regs[0] = t(-1); // exit code
        vm.step().unwrap();
        assert_eq!(vm.exit_code, -1);
        assert!(!vm.running);
    }

    #[test]
    fn test_syscall_print_t() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(2); // PRINT_T
        vm.regs[0] = t(4); // 4 = 0t11
        vm.step().unwrap();
        assert_eq!(vm.output, "11");
        assert!(vm.running); // not halted
    }

    #[test]
    fn test_syscall_print_d() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(3); // PRINT_D
        vm.regs[0] = t(42);
        vm.step().unwrap();
        assert_eq!(vm.output, "42");
        assert!(vm.running);
    }

    #[test]
    fn test_syscall_print_c() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(4); // PRINT_C
        vm.regs[0] = t(65); // 'A'
        vm.step().unwrap();
        assert_eq!(vm.output, "A");
        assert!(vm.running);
    }

    #[test]
    fn test_syscall_print_s() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(7); // PRINT_S
        vm.memory[100] = t(72);   // 'H'
        vm.memory[101] = t(105);  // 'i'
        vm.memory[102] = t(0);    // null
        vm.regs[0] = t(100); // addr
        vm.step().unwrap();
        assert_eq!(vm.output, "Hi");
        assert!(vm.running);
    }

    #[test]
    fn test_syscall_print_s_empty() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(7);
        vm.memory[200] = t(0);
        vm.regs[0] = t(200);
        vm.step().unwrap();
        assert_eq!(vm.output, "");
        assert!(vm.running);
    }

    #[test]
    fn test_syscall_print_s_truncate() {
        let mut vm = VM::new();
        for i in 0..260 {
            vm.memory[i] = t(65); // 'A' — no null within 256 bytes
        }
        place_inst(&mut vm, 500, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(7);
        vm.regs[0] = t(0); // address of string (all 'A')
        vm.pc = 500;
        vm.step().unwrap();
        assert_eq!(vm.output.len(), 256);
        assert!(vm.running);
    }

    #[test]
    fn test_syscall_invalid_func() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(0); // invalid
        vm.step().unwrap();
        assert_eq!(vm.output, "");
        assert!(vm.running); // not halted
    }

    #[test]
    fn test_syscall_accumulates() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Syscall { rs: r(1) });
        place_inst(&mut vm, 2, &Instruction::Syscall { rs: r(1) });
        vm.regs[1] = t(3); // PRINT_D (both calls)
        vm.regs[0] = t(42);
        vm.step().unwrap();
        assert_eq!(vm.output, "42");
        vm.regs[0] = t(50);
        vm.step().unwrap();
        assert_eq!(vm.output, "4250");
        assert!(vm.running);
    }

    // ── C-70: HALT ──

    #[test]
    fn test_halt() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Halt);
        vm.step().unwrap();
        assert!(!vm.running);
    }

    #[test]
    fn test_step_after_halt_returns_halted() {
        let mut vm = VM::new();
        place_inst(&mut vm, 0, &Instruction::Halt);
        vm.step().unwrap();
        assert_eq!(vm.step(), Err(ExecError::Halted));
    }

    // ── C-80: main loop (run) ──

    #[test]
    fn test_run_simple_program() {
        let mut vm = VM::new();
        // Compute 3 + 4 * 2 = 11
        place_inst(&mut vm, 0, &Instruction::Ldi { rd: r(1), imm: t(3) });
        place_inst(&mut vm, 2, &Instruction::Ldi { rd: r(2), imm: t(4) });
        place_inst(&mut vm, 4, &Instruction::Ldi { rd: r(3), imm: t(2) });
        place_inst(&mut vm, 6, &Instruction::Mul { rd: r(0), rs: r(2), rt: r(3) });
        place_inst(&mut vm, 8, &Instruction::Add { rd: r(0), rs: r(1), rt: r(0) });
        place_inst(&mut vm, 10, &Instruction::Halt);
        vm.run();
        assert_eq!(vm.regs[0].to_i16(), 11);
        assert!(!vm.running);
    }

    // ── S1: Fibonacci ──

    #[test]
    fn test_fibonacci() {
        let mut vm = VM::new();
        // Compute fib(5) = 5
        // R1 = a, R2 = b, R4 = counter, R5 = 0
        place_inst(&mut vm, 0, &Instruction::Ldi { rd: r(1), imm: t(0) });  // a=0
        place_inst(&mut vm, 2, &Instruction::Ldi { rd: r(2), imm: t(1) });  // b=1
        place_inst(&mut vm, 4, &Instruction::Ldi { rd: r(4), imm: t(5) });  // counter=5
        place_inst(&mut vm, 6, &Instruction::Ldi { rd: r(5), imm: t(0) });  // zero
        // loop:
        place_inst(&mut vm, 8, &Instruction::Bz { rs: r(4), offset: t(12) }); // to done
        place_inst(&mut vm, 10, &Instruction::Add { rd: r(3), rs: r(1), rt: r(2) }); // tmp=a+b
        place_inst(&mut vm, 12, &Instruction::Add { rd: r(1), rs: r(2), rt: r(5) }); // a=b
        place_inst(&mut vm, 14, &Instruction::Add { rd: r(2), rs: r(3), rt: r(5) }); // b=tmp
        place_inst(&mut vm, 16, &Instruction::AddI { rd: r(4), rs: r(4), imm: t(-1) }); // counter--
        place_inst(&mut vm, 18, &Instruction::Jmp { offset: t(-10) }); // back to loop
        // done:
        place_inst(&mut vm, 20, &Instruction::Halt);

        vm.run();
        assert_eq!(vm.regs[1].to_i16(), 5); // fib(5) in R1
        assert_eq!(vm.regs[2].to_i16(), 8); // fib(6) in R2
        assert!(!vm.running);
    }
}
