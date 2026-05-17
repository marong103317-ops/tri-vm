use crate::instruction::{Instruction, Reg};
use crate::tryte::Tryte;
use std::collections::HashMap;
use std::fmt;

// ── Errors ──

#[derive(Debug, Clone, PartialEq)]
pub enum AssembleError {
    Lex(String),
    Parse(String),
    Codegen(String),
}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssembleError::Lex(msg) => write!(f, "lex error: {}", msg),
            AssembleError::Parse(msg) => write!(f, "parse error: {}", msg),
            AssembleError::Codegen(msg) => write!(f, "codegen error: {}", msg),
        }
    }
}

// ── Token ──

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Register(u8),
    Number(i16),
    Directive(String),
    StringLit(String),
    Comma,
    Colon,
    Newline,
}

// ── Parsed line types ──

#[derive(Debug, Clone, PartialEq)]
enum Operand {
    Reg(u8),
    Imm(i16),
    Label(String),
}

#[derive(Debug, Clone)]
enum LineKind {
    Text {
        mnemonic: String,
        operands: Vec<Operand>,
    },
    Word(Vec<i16>),
    Asciiz(String),
}

#[derive(Debug, Clone)]
struct Line {
    label: Option<String>,
    kind: LineKind,
}

// ── Lexer ──

fn lex(source: &str) -> Result<Vec<Token>, AssembleError> {
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let len = chars.len();
    let mut tokens = Vec::new();

    while i < len {
        let c = chars[i];

        // whitespace (except \n)
        if c.is_ascii_whitespace() && c != '\n' {
            i += 1;
            continue;
        }

        // newline
        if c == '\n' {
            tokens.push(Token::Newline);
            i += 1;
            continue;
        }

        // comment
        if c == ';' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // comma
        if c == ',' {
            tokens.push(Token::Comma);
            i += 1;
            continue;
        }

        // colon
        if c == ':' {
            tokens.push(Token::Colon);
            i += 1;
            continue;
        }

        // string literal
        if c == '"' {
            let mut s = String::new();
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '0' => s.push('\0'),
                        _ => {
                            return Err(AssembleError::Lex(format!(
                                "invalid escape: \\{}",
                                chars[i]
                            )))
                        }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i >= len {
                return Err(AssembleError::Lex("unterminated string literal".into()));
            }
            i += 1; // skip closing "
            tokens.push(Token::StringLit(s));
            continue;
        }

        // directive (starts with .)
        if c == '.' {
            let mut name = String::new();
            i += 1; // skip '.'
            while i < len && chars[i].is_ascii_alphabetic() {
                name.push(chars[i]);
                i += 1;
            }
            if name.is_empty() {
                return Err(AssembleError::Lex("bare '.'".into()));
            }
            tokens.push(Token::Directive(name));
            continue;
        }

        // number or identifier
        if c.is_ascii_alphanumeric() || c == '+' || c == '-' {
            let mut word = String::new();
            // include leading sign for numbers
            if (c == '+' || c == '-') && i + 1 < len && chars[i + 1].is_ascii_digit() {
                word.push(c);
                i += 1;
            } else if c == '+' || c == '-' {
                // sign not followed by digit — treat as identifier part?
                // For now, treat as ident
                word.push(c);
                i += 1;
            }

            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]);
                i += 1;
            }

            // Classify word: number or identifier
            if let Some(val) = parse_number(&word) {
                tokens.push(Token::Number(val));
            } else if word.len() == 2
                && (word == "R0" || word == "R1" || word == "R2" || word == "R3"
                    || word == "R4" || word == "R5" || word == "R6" || word == "R7")
            {
                let reg = word[1..].parse::<u8>().unwrap();
                tokens.push(Token::Register(reg));
            } else {
                tokens.push(Token::Ident(word));
            }
            continue;
        }

        return Err(AssembleError::Lex(format!("unexpected character '{}'", c)));
    }

    Ok(tokens)
}

fn parse_number(word: &str) -> Option<i16> {
    // 0x prefix → hex
    if let Some(hex) = word.strip_prefix("0x").or_else(|| word.strip_prefix("0X")) {
        return i16::from_str_radix(hex, 16).ok().filter(|&v| v >= -364 && v <= 364);
    }
    // 0b prefix → binary
    if let Some(bin) = word.strip_prefix("0b").or_else(|| word.strip_prefix("0B")) {
        return i16::from_str_radix(bin, 2).ok().filter(|&v| v >= -364 && v <= 364);
    }
    // 0t prefix → balanced ternary
    if let Some(tern) = word.strip_prefix("0t").or_else(|| word.strip_prefix("0T")) {
        return parse_balanced_ternary(tern);
    }
    // bare balanced ternary (must contain at least one T/t, otherwise pure digits are decimal)
    if !word.is_empty()
        && word.chars().any(|c| c == 'T' || c == 't')
        && word.chars().all(|c| c == 'T' || c == 't' || c == '0' || c == '1')
    {
        return parse_balanced_ternary(word);
    }
    // decimal
    if let Ok(v) = word.parse::<i16>() {
        if v >= -364 && v <= 364 {
            return Some(v);
        }
    }
    None
}

fn parse_balanced_ternary(s: &str) -> Option<i16> {
    let mut val: i16 = 0;
    let mut mul: i16 = 1;
    let chars: Vec<char> = s.chars().collect();
    for &c in chars.iter().rev() {
        let digit = match c {
            '1' => 1i16,
            '0' => 0,
            'T' | 't' => -1,
            _ => return None,
        };
        val = val.checked_add(digit.checked_mul(mul)?)?;
        mul = mul.checked_mul(3)?;
    }
    if val >= -364 && val <= 364 {
        Some(val)
    } else {
        None
    }
}

// ── Parser ──

fn parse(tokens: &[Token]) -> Result<Vec<Line>, AssembleError> {
    let mut lines: Vec<Line> = Vec::new();
    let mut in_text = true;
    let mut i = 0;
    let len = tokens.len();
    let mut pending_label: Option<String> = None;

    while i < len {
        match &tokens[i] {
            Token::Directive(name) if name == "text" => {
                in_text = true;
                i += 1;
                if i < len && tokens[i] == Token::Newline {
                    i += 1;
                }
                continue;
            }
            Token::Directive(name) if name == "data" => {
                in_text = false;
                i += 1;
                if i < len && tokens[i] == Token::Newline {
                    i += 1;
                }
                continue;
            }
            Token::Newline => {
                i += 1;
                continue;
            }
            _ => {}
        }

        // Try to parse a label at this position
        let captured_label: Option<String>;
        let start: usize;
        if i + 1 < len && tokens[i + 1] == Token::Colon {
            if let Token::Ident(name) = &tokens[i] {
                if pending_label.is_some() {
                    return Err(AssembleError::Parse(format!(
                        "consecutive labels: '{}' overwrites previous label",
                        name
                    )));
                }
                let label_name = name.clone();
                i += 2; // skip ident and colon
                pending_label = Some(label_name);
                continue;
            } else {
                return Err(AssembleError::Parse(
                    "label must be an identifier".into(),
                ));
            }
        } else if let Some(pl) = pending_label.take() {
            captured_label = Some(pl);
            start = i;
        } else {
            captured_label = None;
            start = i;
        }

        if start >= len || tokens[start] == Token::Newline {
            i = start + 1;
            continue;
        }

        if in_text {
            let (line, end) = parse_text_line(tokens, start, captured_label)?;
            lines.push(line);
            i = end;
        } else {
            let (line, end) = parse_data_line(tokens, start, captured_label)?;
            lines.push(line);
            i = end;
        }
    }

    if let Some(label) = pending_label {
        return Err(AssembleError::Parse(format!(
            "label '{}' at end of file without instruction or directive",
            label
        )));
    }

    Ok(lines)
}

/// Parse a text line starting at position i. Returns (Line, next_position).
fn parse_text_line(
    tokens: &[Token],
    i: usize,
    label: Option<String>,
) -> Result<(Line, usize), AssembleError> {
    let mnemonic = match &tokens[i] {
        Token::Ident(m) => m.clone(),
        _ => {
            return Err(AssembleError::Parse(format!(
                "expected instruction mnemonic, got {:?}",
                tokens[i]
            )))
        }
    };

    let mut j = i + 1;
    let mut operands = Vec::new();

    // skip Newline tokens between operands (allow multi-line)
    while j < tokens.len() && tokens[j] != Token::Newline {
        match &tokens[j] {
            Token::Register(r) => {
                operands.push(Operand::Reg(*r));
                j += 1;
            }
            Token::Number(n) => {
                operands.push(Operand::Imm(*n));
                j += 1;
            }
            Token::Ident(name) => {
                // label reference
                operands.push(Operand::Label(name.clone()));
                j += 1;
            }
            Token::Comma => {
                j += 1;
            }
            _ => {
                return Err(AssembleError::Parse(format!(
                    "unexpected token {:?} in instruction operands",
                    tokens[j]
                )))
            }
        }
    }

    let end = j + 1; // skip Newline

    Ok((
        Line {
            label,
            kind: LineKind::Text { mnemonic, operands },
        },
        end,
    ))
}

/// Parse a data line starting at position i. Returns (Line, next_position).
fn parse_data_line(
    tokens: &[Token],
    i: usize,
    label: Option<String>,
) -> Result<(Line, usize), AssembleError> {
    let directive = match &tokens[i] {
        Token::Directive(d) => d.clone(),
        _ => {
            return Err(AssembleError::Parse(format!(
                "expected directive in .data section, got {:?}",
                tokens[i]
            )))
        }
    };

    let mut j = i + 1;
    match directive.as_str() {
        "word" => {
            let mut values = Vec::new();
            while j < tokens.len() && tokens[j] != Token::Newline {
                match &tokens[j] {
                    Token::Number(n) => values.push(*n),
                    Token::Ident(name) => {
                        // could be a label reference for address
                        return Err(AssembleError::Parse(format!(
                            "unresolved label '{}' in .word",
                            name
                        )));
                    }
                    Token::Comma => {}
                    _ => {
                        return Err(AssembleError::Parse(format!(
                            "unexpected token {:?} in .word",
                            tokens[j]
                        )))
                    }
                }
                j += 1;
            }
            let end = j + 1;
            Ok((Line { label, kind: LineKind::Word(values) }, end))
        }
        "asciiz" => {
            let s = match &tokens[j] {
                Token::StringLit(s) => s.clone(),
                _ => {
                    return Err(AssembleError::Parse(format!(
                        "expected string literal after .asciiz, got {:?}",
                        tokens[j]
                    )))
                }
            };
            j += 1;
            let end = j + 1; // skip newline
            Ok((Line { label, kind: LineKind::Asciiz(s) }, end))
        }
        other => Err(AssembleError::Parse(format!(
            "unknown data directive '{}'",
            other
        ))),
    }
}

// ── Codegen ──

fn codegen(lines: &[Line]) -> Result<Vec<Tryte>, AssembleError> {
    // First pass: assign addresses and build symbol table
    let mut symbols: HashMap<String, i32> = HashMap::new();
    let mut addr = 0i32;

    struct AddrLine {
        addr: i32,
        kind: LineKind,
    }
    let mut addr_lines: Vec<AddrLine> = Vec::new();

    for line in lines {
        if let Some(label) = &line.label {
            if symbols.contains_key(label) {
                return Err(AssembleError::Codegen(format!(
                    "duplicate label '{}'",
                    label
                )));
            }
            symbols.insert(label.clone(), addr);
        }
        match &line.kind {
            LineKind::Text { .. } => {
                addr_lines.push(AddrLine {
                    addr,
                    kind: line.kind.clone(),
                });
                addr += 2;
            }
            LineKind::Word(vals) => {
                addr_lines.push(AddrLine {
                    addr,
                    kind: line.kind.clone(),
                });
                addr += vals.len() as i32;
            }
            LineKind::Asciiz(s) => {
                addr_lines.push(AddrLine {
                    addr,
                    kind: line.kind.clone(),
                });
                addr += s.len() as i32 + 1;
            }
        }
    }

    // Second pass: emit trytes
    let mut binary: Vec<Tryte> = Vec::new();

    for AddrLine { addr, kind } in &addr_lines {
        match kind {
            LineKind::Text {
                mnemonic,
                operands,
            } => {
                let inst = resolve_instruction(mnemonic, operands, &symbols, *addr)?;
                let (hi, lo) = crate::instruction::encode(&inst);
                binary.push(hi);
                binary.push(lo);
            }
            LineKind::Word(vals) => {
                for v in vals {
                    let t =
                        Tryte::from_i16(*v).ok_or_else(|| {
                            AssembleError::Codegen(format!("word value {} out of range", v))
                        })?;
                    binary.push(t);
                }
            }
            LineKind::Asciiz(s) => {
                for &ch in s.as_bytes() {
                    let t = Tryte::from_i16(ch as i16).ok_or_else(|| {
                        AssembleError::Codegen(format!("char {} out of range", ch))
                    })?;
                    binary.push(t);
                }
                binary.push(Tryte::ZERO); // null terminator
            }
        }
    }

    Ok(binary)
}

fn resolve_instruction(
    mnemonic: &str,
    operands: &[Operand],
    symbols: &HashMap<String, i32>,
    addr: i32,
) -> Result<Instruction, AssembleError> {
    match mnemonic {
        "NOP" => Ok(Instruction::Nop),

        "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "MAC" | "CMP" => {
            if operands.len() != 3 {
                return Err(AssembleError::Codegen(format!(
                    "{} requires 3 register operands, got {}",
                    mnemonic,
                    operands.len()
                )));
            }
            let rd = reg_operand(&operands[0])?;
            let rs = reg_operand(&operands[1])?;
            let rt = reg_operand(&operands[2])?;
            match mnemonic {
                "ADD" => Ok(Instruction::Add { rd, rs, rt }),
                "SUB" => Ok(Instruction::Sub { rd, rs, rt }),
                "MUL" => Ok(Instruction::Mul { rd, rs, rt }),
                "DIV" => Ok(Instruction::Div { rd, rs, rt }),
                "MOD" => Ok(Instruction::Modulo { rd, rs, rt }),
                "MAC" => Ok(Instruction::Mac { rd, rs, rt }),
                "CMP" => Ok(Instruction::Cmp { rd, rs, rt }),
                _ => unreachable!(),
            }
        }

        "SGN" => {
            if operands.len() != 2 {
                return Err(AssembleError::Codegen(format!(
                    "SGN requires 2 register operands, got {}",
                    operands.len()
                )));
            }
            let rd = reg_operand(&operands[0])?;
            let rs = reg_operand(&operands[1])?;
            Ok(Instruction::Sgn { rd, rs })
        }

        "ADDI" | "MULI" | "LD" | "ST" => {
            if operands.len() != 3 {
                return Err(AssembleError::Codegen(format!(
                    "{} requires reg, reg, imm, got {} operands",
                    mnemonic,
                    operands.len()
                )));
            }
            let rd = reg_operand(&operands[0])?;
            let rs = reg_operand(&operands[1])?;
            let imm = imm_operand(&operands[2], symbols, addr)?;
            match mnemonic {
                "ADDI" => Ok(Instruction::AddI { rd, rs, imm }),
                "MULI" => Ok(Instruction::MulI { rd, rs, imm }),
                "LD" => Ok(Instruction::Ld { rd, rs, imm }),
                "ST" => Ok(Instruction::St { rd, rs, imm }),
                _ => unreachable!(),
            }
        }

        "LDI" => {
            if operands.len() != 2 {
                return Err(AssembleError::Codegen(format!(
                    "LDI requires reg, imm, got {} operands",
                    operands.len()
                )));
            }
            let rd = reg_operand(&operands[0])?;
            let imm = imm_operand(&operands[1], symbols, addr)?;
            Ok(Instruction::Ldi { rd, imm })
        }

        "JMP" | "CALL" => {
            if operands.len() != 1 {
                return Err(AssembleError::Codegen(format!(
                    "{} requires 1 operand, got {}",
                    mnemonic,
                    operands.len()
                )));
            }
            let offset = offset_operand(&operands[0], symbols, addr)?;
            match mnemonic {
                "JMP" => Ok(Instruction::Jmp { offset }),
                "CALL" => Ok(Instruction::Call { offset }),
                _ => unreachable!(),
            }
        }

        "JMPR" | "SYSCALL" => {
            if operands.len() != 1 {
                return Err(AssembleError::Codegen(format!(
                    "{} requires 1 register operand, got {}",
                    mnemonic,
                    operands.len()
                )));
            }
            let rs = reg_operand(&operands[0])?;
            match mnemonic {
                "JMPR" => Ok(Instruction::JmpR { rs }),
                "SYSCALL" => Ok(Instruction::Syscall { rs }),
                _ => unreachable!(),
            }
        }

        "BZ" | "BN" | "BP" => {
            if operands.len() != 2 {
                return Err(AssembleError::Codegen(format!(
                    "{} requires reg, offset, got {} operands",
                    mnemonic,
                    operands.len()
                )));
            }
            let rs = reg_operand(&operands[0])?;
            let offset = offset_operand(&operands[1], symbols, addr)?;
            match mnemonic {
                "BZ" => Ok(Instruction::Bz { rs, offset }),
                "BN" => Ok(Instruction::Bn { rs, offset }),
                "BP" => Ok(Instruction::Bp { rs, offset }),
                _ => unreachable!(),
            }
        }

        "RET" => {
            if !operands.is_empty() {
                return Err(AssembleError::Codegen(format!(
                    "RET takes no operands, got {}",
                    operands.len()
                )));
            }
            Ok(Instruction::Ret)
        }
        "HALT" => {
            if !operands.is_empty() {
                return Err(AssembleError::Codegen(format!(
                    "HALT takes no operands, got {}",
                    operands.len()
                )));
            }
            Ok(Instruction::Halt)
        }

        _ => Err(AssembleError::Codegen(format!(
            "unknown mnemonic '{}'",
            mnemonic
        ))),
    }
}

fn reg_operand(op: &Operand) -> Result<Reg, AssembleError> {
    match op {
        Operand::Reg(r) => Ok(Reg(*r)),
        _ => Err(AssembleError::Codegen(format!(
            "expected register, got {:?}",
            op
        ))),
    }
}

fn imm_operand(
    op: &Operand,
    symbols: &HashMap<String, i32>,
    _addr: i32,
) -> Result<Tryte, AssembleError> {
    let val = match op {
        Operand::Imm(v) => *v,
        Operand::Label(name) => {
            let target = symbols
                .get(name)
                .ok_or_else(|| AssembleError::Codegen(format!("undefined label '{}'", name)))?;
            *target as i16
        }
        Operand::Reg(_) => {
            return Err(AssembleError::Codegen("expected immediate, got register".into()))
        }
    };
    Tryte::from_i16(val)
        .ok_or_else(|| AssembleError::Codegen(format!("immediate value {} out of range", val)))
}

fn offset_operand(
    op: &Operand,
    symbols: &HashMap<String, i32>,
    addr: i32,
) -> Result<Tryte, AssembleError> {
    let target = match op {
        Operand::Imm(v) => return Ok(Tryte::from_i16(*v).ok_or_else(|| {
            AssembleError::Codegen(format!("offset {} out of range", v))
        })?),
        Operand::Label(name) => symbols.get(name).ok_or_else(|| {
            AssembleError::Codegen(format!("undefined label '{}'", name))
        })?,
        Operand::Reg(_) => {
            return Err(AssembleError::Codegen("expected offset, got register".into()))
        }
    };

    // offset = target - current_addr (PC points at instruction, not advanced)
    let offset = target - addr;
    Tryte::from_i16(offset as i16).ok_or_else(|| {
        AssembleError::Codegen(format!(
            "offset from {} to {} = {} out of range (-364..364)",
            addr, target, offset
        ))
    })
}

// ── Public API ──

pub fn assemble(source: &str) -> Result<Vec<Tryte>, AssembleError> {
    let tokens = lex(source)?;
    let lines = parse(&tokens)?;
    codegen(&lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    fn t(v: i16) -> Tryte {
        Tryte::from_i16(v).unwrap()
    }

    // ── Lexer tests ──

    #[test]
    fn test_lex_simple_instruction() {
        let tokens = lex("ADD R0, R1, R2\n").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("ADD".into()),
                Token::Register(0),
                Token::Comma,
                Token::Register(1),
                Token::Comma,
                Token::Register(2),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_lex_ldi_with_number() {
        let tokens = lex("LDI R0, 42\n").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("LDI".into()),
                Token::Register(0),
                Token::Comma,
                Token::Number(42),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_lex_ldi_with_neg_number() {
        let tokens = lex("LDI R0, -13\n").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("LDI".into()),
                Token::Register(0),
                Token::Comma,
                Token::Number(-13),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_lex_comment() {
        let tokens = lex("ADD R0, R1, R2 ; this is a comment\n").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("ADD".into()),
                Token::Register(0),
                Token::Comma,
                Token::Register(1),
                Token::Comma,
                Token::Register(2),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_lex_label() {
        let tokens = lex("loop:\n").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Ident("loop".into()), Token::Colon, Token::Newline]
        );
    }

    #[test]
    fn test_lex_directive() {
        let tokens = lex(".data\n").unwrap();
        assert_eq!(tokens, vec![Token::Directive("data".into()), Token::Newline]);
    }

    #[test]
    fn test_lex_string_literal() {
        let tokens = lex(".asciiz \"Hello\"\n").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Directive("asciiz".into()),
                Token::StringLit("Hello".into()),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn test_lex_balanced_ternary() {
        let tokens = lex("0t1T0\n").unwrap();
        assert_eq!(tokens, vec![Token::Number(6), Token::Newline]);
    }

    #[test]
    fn test_lex_standalone_t() {
        let tokens = lex("T\n").unwrap();
        assert_eq!(tokens, vec![Token::Number(-1), Token::Newline]);
    }

    #[test]
    fn test_lex_hex_number() {
        let tokens = lex("0xFF\n").unwrap();
        assert_eq!(tokens, vec![Token::Number(255), Token::Newline]);
    }

    #[test]
    fn test_lex_binary_number() {
        let tokens = lex("0b1010\n").unwrap();
        assert_eq!(tokens, vec![Token::Number(10), Token::Newline]);
    }

    #[test]
    fn test_lex_register_r0_to_r7() {
        let tokens = lex("R0 R1 R2 R3 R4 R5 R6 R7\n").unwrap();
        assert_eq!(tokens.len(), 9); // 8 regs + 1 newline
        for i in 0..8 {
            assert_eq!(tokens[i], Token::Register(i as u8));
        }
    }

    // ── Parser tests ──

    #[test]
    fn test_parse_add() {
        let lines = parse(&lex("ADD R0, R1, R2\n").unwrap()).unwrap();
        assert_eq!(lines.len(), 1);
        match &lines[0].kind {
            LineKind::Text { mnemonic, operands } => {
                assert_eq!(mnemonic, "ADD");
                assert_eq!(operands.len(), 3);
                assert_eq!(operands[0], Operand::Reg(0));
                assert_eq!(operands[1], Operand::Reg(1));
                assert_eq!(operands[2], Operand::Reg(2));
            }
            _ => panic!("expected text line"),
        }
    }

    #[test]
    fn test_parse_label_and_instruction() {
        let src = "loop:\n  LDI R0, 42\n";
        let lines = parse(&lex(src).unwrap()).unwrap();
        assert_eq!(lines.len(), 1);
        // The bare label binds to the following LDI
        assert_eq!(lines[0].label.as_deref(), Some("loop"));
        match &lines[0].kind {
            LineKind::Text { mnemonic, operands } => {
                assert_eq!(mnemonic, "LDI");
                assert_eq!(operands, &[Operand::Reg(0), Operand::Imm(42)]);
            }
            _ => panic!("expected text line"),
        }
    }

    #[test]
    fn test_parse_jmp_to_label() {
        let src = "JMP done\n";
        let lines = parse(&lex(src).unwrap()).unwrap();
        match &lines[0].kind {
            LineKind::Text { mnemonic, operands } => {
                assert_eq!(mnemonic, "JMP");
                assert_eq!(operands[0], Operand::Label("done".into()));
            }
            _ => panic!("expected text line"),
        }
    }

    #[test]
    fn test_parse_data_section() {
        // bare label on its own line, then directive
        let src = ".data\nmsg:\n  .asciiz \"Hello\"\n";
        let lines = parse(&lex(src).unwrap()).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].label.as_deref(), Some("msg"));
        match &lines[0].kind {
            LineKind::Asciiz(s) => assert_eq!(s, "Hello"),
            _ => panic!("expected Asciiz"),
        }
        // label on same line as directive
        let src2 = ".data\nmsg: .asciiz \"Hello\"\n";
        let lines2 = parse(&lex(src2).unwrap()).unwrap();
        assert_eq!(lines2.len(), 1);
        assert_eq!(lines2[0].label.as_deref(), Some("msg"));
        match &lines2[0].kind {
            LineKind::Asciiz(s) => assert_eq!(s, "Hello"),
            _ => panic!("expected Asciiz"),
        }
    }

    #[test]
    fn test_parse_word() {
        let src = ".data\n.word 1, -2, 3\n";
        let lines = parse(&lex(src).unwrap()).unwrap();
        assert_eq!(lines.len(), 1);
        // The first line may not have label since `.data` doesn't set label
        match &lines[0].kind {
            LineKind::Word(vals) => {
                assert_eq!(vals, &[1, -2, 3]);
            }
            _ => panic!("expected Word"),
        }
    }

    // ── Codegen tests ──

    #[test]
    fn test_assemble_ldi() {
        let binary = assemble("LDI R0, 42\nLDI R1, -13\nHALT\n").unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.regs[0].to_i16(), 42);
        assert_eq!(vm.regs[1].to_i16(), -13);
    }

    #[test]
    fn test_assemble_simple_program() {
        // Compute 3 + 4 * 2 = 11
        let src = "\
LDI R1, 3
LDI R2, 4
LDI R3, 2
MUL R0, R2, R3
ADD R0, R1, R0
HALT
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.regs[0].to_i16(), 11);
    }

    #[test]
    fn test_assemble_with_label_jump() {
        let src = "\
LDI R0, 0
LDI R1, 1
JMP skip
LDI R0, 99
skip:
HALT
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        // R0 should be 0 (not 99), because JMP skipped the LDI R0, 99
        assert_eq!(vm.regs[0].to_i16(), 0);
        assert_eq!(vm.regs[1].to_i16(), 1);
    }

    #[test]
    fn test_assemble_bz_branch() {
        let src = "\
LDI R0, 5
BZ R0, skip
LDI R0, 99
skip:
HALT
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        // R0 != 0, so BZ not taken, R0 should be 99
        assert_eq!(vm.regs[0].to_i16(), 99);
    }

    #[test]
    fn test_assemble_bz_taken() {
        let src = "\
LDI R0, 0
BZ R0, skip
LDI R0, 99
skip:
HALT
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        // R0 == 0, so BZ is taken, R0 remains 0
        assert_eq!(vm.regs[0].to_i16(), 0);
    }

    #[test]
    fn test_assemble_data_word() {
        let src = "\
.text
LDI R0, val
LD R1, R0, 0
HALT
.data
val: .word 42
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.regs[1].to_i16(), 42);
    }

    #[test]
    fn test_assemble_asciiz() {
        let src = "\
.text
LDI R0, msg
SYSCALL R1
HALT
.data
msg: .asciiz \"Hi\"
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.regs[1] = t(7); // PRINT_S
        vm.run();
        assert_eq!(vm.output, "Hi");
    }

    // ── Fibonacci integration test ──

    #[test]
    fn test_assemble_fibonacci() {
        let src = "\
.text
  LDI R1, 0        ; a = 0
  LDI R2, 1        ; b = 1
  LDI R4, 5        ; counter = N
  LDI R5, 0        ; zero register
loop:
  BZ R4, done      ; if counter == 0 -> done
  ADD R3, R1, R2   ; tmp = a + b
  ADD R1, R2, R5   ; a = b
  ADD R2, R3, R5   ; b = tmp
  ADDI R4, R4, -1  ; counter--
  JMP loop
done:
  HALT
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.regs[1].to_i16(), 5); // fib(5) in R1
        assert_eq!(vm.regs[2].to_i16(), 8); // fib(6) in R2
        assert!(!vm.running);
    }

    // ── Error tests ──

    #[test]
    fn test_assemble_undefined_label() {
        let src = "JMP nowhere\nHALT\n";
        let result = assemble(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("undefined label"));
    }

    #[test]
    fn test_assemble_duplicate_label() {
        let src = "loop: NOP\nloop: NOP\n";
        let result = assemble(src);
        assert!(result.is_err());
    }

    #[test]
    fn test_lex_unterminated_string() {
        let result = lex(".asciiz \"unterminated\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_lex_bare_dot() {
        let result = lex(".\n");
        assert!(result.is_err());
    }

    // ── Regression: bare T = -1, 1T0 = 6, but pure digits stay decimal ──

    #[test]
    fn test_parse_number_t_is_neg_one() {
        // T alone should be -1 (balanced ternary)
        assert_eq!(parse_number("T"), Some(-1));
        assert_eq!(parse_number("t"), Some(-1));
    }

    #[test]
    fn test_parse_number_ternary_string() {
        // 1T0 should be 6 (balanced ternary)
        assert_eq!(parse_number("1T0"), Some(6));
        assert_eq!(parse_number("0t1T0"), Some(6));
    }

    #[test]
    fn test_parse_number_pure_digits_stay_decimal() {
        // 10 must be decimal 10, NOT balanced ternary (which would be 3)
        assert_eq!(parse_number("10"), Some(10));
        assert_eq!(parse_number("11"), Some(11));
        assert_eq!(parse_number("100"), Some(100));
    }

    #[test]
    fn test_assemble_decimal_ten_not_ternary() {
        // LDI with 10 should load decimal 10, not ternary 3
        let binary = assemble("LDI R0, 10\nHALT\n").unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.regs[0].to_i16(), 10);
    }

    // ── Regression: consecutive bare labels ──

    #[test]
    fn test_consecutive_labels_error() {
        let result = assemble("start:\nloop:\n  NOP\n");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("consecutive labels"));
    }

    // ── Regression: HALT/RET with operands ──

    #[test]
    fn test_halt_with_operand_error() {
        let result = assemble("HALT R0\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_ret_with_operand_error() {
        let result = assemble("RET R1\n");
        assert!(result.is_err());
    }

    // ── Regression: label at end of file ──

    #[test]
    fn test_label_at_eof_error() {
        let result = assemble("LDI R0, 0\ndangling:\n");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("end of file"));
    }

    // ── CALL/RET with labels ──

    #[test]
    fn test_assemble_call_ret() {
        let src = "\
.text
  LDI R0, 5
  CALL square
  HALT
square:
  MUL R0, R0, R0
  RET
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.regs[0].to_i16(), 25);
    }

    #[test]
    fn test_sgn_zero_prints_zero() {
        let src = "\
LDI R0, 0
SGN R0, R0
LDI R1, 3
SYSCALL R1
LDI R1, 1
SYSCALL R1
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.output, "0");
    }

    #[test]
    fn test_store_load_addr_200() {
        let src = "\
LDI R5, 0
LDI R0, 42
LDI R6, 200
ST R0, R6, 0
LDI R6, 200
LD R0, R6, 0
LDI R1, 3
SYSCALL R1
LDI R1, 1
SYSCALL R1
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.output, "42");
        assert_eq!(vm.exit_code, 42);
    }

    #[test]
    fn test_tnn_one_neuron_zero_input() {
        let src = "\
LDI R5, ly1_w00
LDI R1, 0
LDI R2, 0
LDI R3, 0
LDI R4, 0
LDI R0, 0
LD R6, R5, 0
MAC R0, R1, R6
ADDI R5, R5, 1
LD R6, R5, 0
MAC R0, R2, R6
ADDI R5, R5, 1
LD R6, R5, 0
MAC R0, R3, R6
ADDI R5, R5, 1
LD R6, R5, 0
MAC R0, R4, R6
ADDI R5, R5, 1
LD R6, R5, 0
ADD R0, R0, R6
LDI R1, 3
SYSCALL R1
LDI R0, 0
LDI R1, 1
SYSCALL R1
.data
ly1_w00: .word 1
ly1_w01: .word 0
ly1_w02: .word T
ly1_w03: .word 0
ly1_b0:  .word 0
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.output, "0");
    }

    #[test]
    fn test_tnn_inference_all_positive() {
        // Input [1,1,T,T] → output 1  (tnn.tri 默认测试向量)
        let source = include_str!("../examples/tnn.tri");
        let binary = assemble(source).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.output, "1\n");
    }

    #[test]
    fn test_tnn_all_ones() {
        // Input [1,1,1,1] → output 0
        let src = "\
.text
LDI R1, 1
LDI R2, 1
LDI R3, 1
LDI R4, 1
LDI R5, ly1_w00
CALL fc_neuron
SGN R0, R0
LDI R6, 200
ST R0, R6, 0
LDI R5, ly1_w10
CALL fc_neuron
SGN R0, R0
LDI R6, 201
ST R0, R6, 0
LDI R5, ly1_w20
CALL fc_neuron
SGN R0, R0
LDI R6, 202
ST R0, R6, 0
LDI R5, ly1_w30
CALL fc_neuron
SGN R0, R0
LDI R6, 203
ST R0, R6, 0
LDI R6, 200
LD R1, R6, 0
LDI R6, 201
LD R2, R6, 0
LDI R6, 202
LD R3, R6, 0
LDI R6, 203
LD R4, R6, 0
LDI R5, ly2_w00
CALL fc_neuron
SGN R0, R0
LDI R1, 3
SYSCALL R1
LDI R0, 10
LDI R1, 4
SYSCALL R1
LDI R0, 0
LDI R1, 1
SYSCALL R1
fc_neuron:
LDI R0, 0
LD R6, R5, 0
MAC R0, R1, R6
ADDI R5, R5, 1
LD R6, R5, 0
MAC R0, R2, R6
ADDI R5, R5, 1
LD R6, R5, 0
MAC R0, R3, R6
ADDI R5, R5, 1
LD R6, R5, 0
MAC R0, R4, R6
ADDI R5, R5, 1
LD R6, R5, 0
ADD R0, R0, R6
RET
.data
ly1_w00: .word 1
ly1_w01: .word 0
ly1_w02: .word T
ly1_w03: .word 0
ly1_b0:  .word 0
ly1_w10: .word 0
ly1_w11: .word 1
ly1_w12: .word 0
ly1_w13: .word T
ly1_b1:  .word 0
ly1_w20: .word T
ly1_w21: .word 0
ly1_w22: .word 1
ly1_w23: .word 0
ly1_b2:  .word 0
ly1_w30: .word 0
ly1_w31: .word T
ly1_w32: .word 0
ly1_w33: .word 1
ly1_b3:  .word 0
ly2_w00: .word 1
ly2_w01: .word 1
ly2_w02: .word T
ly2_w03: .word T
ly2_b0:  .word 0
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.output, "0\n");
    }

    #[test]
    fn test_tnn_inference_all_zero() {
        // Input [0,0,0,0] → output 0
        let src = "\
.text
  LDI R1, 0
  LDI R2, 0
  LDI R3, 0
  LDI R4, 0
  LDI R5, ly1_w00
  CALL fc_neuron
  SGN R0, R0
  LDI R6, 200
  ST R0, R6, 0
  LDI R5, ly1_w10
  CALL fc_neuron
  SGN R0, R0
  LDI R6, 201
  ST R0, R6, 0
  LDI R5, ly1_w20
  CALL fc_neuron
  SGN R0, R0
  LDI R6, 202
  ST R0, R6, 0
  LDI R5, ly1_w30
  CALL fc_neuron
  SGN R0, R0
  LDI R6, 203
  ST R0, R6, 0
  LDI R6, 200
  LD R1, R6, 0
  LDI R6, 201
  LD R2, R6, 0
  LDI R6, 202
  LD R3, R6, 0
  LDI R6, 203
  LD R4, R6, 0
  LDI R5, ly2_w00
  CALL fc_neuron
  SGN R0, R0
  LDI R1, 3
  SYSCALL R1
  LDI R0, 10
  LDI R1, 4
  SYSCALL R1
  LDI R0, 0
  LDI R1, 1
  SYSCALL R1
fc_neuron:
  LDI R0, 0
  LD R6, R5, 0
  MAC R0, R1, R6
  ADDI R5, R5, 1
  LD R6, R5, 0
  MAC R0, R2, R6
  ADDI R5, R5, 1
  LD R6, R5, 0
  MAC R0, R3, R6
  ADDI R5, R5, 1
  LD R6, R5, 0
  MAC R0, R4, R6
  ADDI R5, R5, 1
  LD R6, R5, 0
  ADD R0, R0, R6
  RET
.data
ly1_w00: .word 1
ly1_w01: .word 0
ly1_w02: .word T
ly1_w03: .word 0
ly1_b0:  .word 0
ly1_w10: .word 0
ly1_w11: .word 1
ly1_w12: .word 0
ly1_w13: .word T
ly1_b1:  .word 0
ly1_w20: .word T
ly1_w21: .word 0
ly1_w22: .word 1
ly1_w23: .word 0
ly1_b2:  .word 0
ly1_w30: .word 0
ly1_w31: .word T
ly1_w32: .word 0
ly1_w33: .word 1
ly1_b3:  .word 0
ly2_w00: .word 1
ly2_w01: .word 1
ly2_w02: .word T
ly2_w03: .word T
ly2_b0:  .word 0
";
        let binary = assemble(src).unwrap();
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            vm.memory[i] = *t;
        }
        vm.run();
        assert_eq!(vm.output, "0\n");
    }
}
