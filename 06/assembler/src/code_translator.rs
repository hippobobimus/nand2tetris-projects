use crate::error::{Error, ErrorKind, Result};

/// Translates the 'dest' mnemonic in a C-command into its 16-bit binary
/// representation.
///
/// # Examples
/// "
/// '''
/// assert_eq!(dest("AM").unwrap(), 0b101);
/// '''
pub fn dest(mnemonic: &str) -> Result<u16> {
    match mnemonic {
        "null" => Ok(0b000 << 3),
        "M" => Ok(0b001 << 3),
        "D" => Ok(0b010 << 3),
        "MD" => Ok(0b011 << 3),
        "A" => Ok(0b100 << 3),
        "AM" => Ok(0b101 << 3),
        "AD" => Ok(0b110 << 3),
        "AMD" => Ok(0b111 << 3),
        _ => Err(Error::new(ErrorKind::InvalidSyntax)),
    }
}

/// Translates the 'comp' mnemonic in a C-command into its 16-bit binary
/// representation.
///
/// # Examples
/// "
/// '''
/// assert_eq!(comp("D+A").unwrap(), 0b0000010);
/// '''
pub fn comp(mnemonic: &str) -> Result<u16> {
    match mnemonic {
        "0" => Ok(0b0101010 << 6),
        "1" => Ok(0b0111111 << 6),
        "-1" => Ok(0b0111010 << 6),
        "D" => Ok(0b0001100 << 6),
        "A" => Ok(0b0110000 << 6),
        "!D" => Ok(0b0001101 << 6),
        "!A" => Ok(0b0110001 << 6),
        "-D" => Ok(0b0001111 << 6),
        "-A" => Ok(0b0110011 << 6),
        "D+1" => Ok(0b0011111 << 6),
        "A+1" => Ok(0b0110111 << 6),
        "D-1" => Ok(0b0001110 << 6),
        "A-1" => Ok(0b0110010 << 6),
        "D+A" => Ok(0b0000010 << 6),
        "D-A" => Ok(0b0010011 << 6),
        "A-D" => Ok(0b0000111 << 6),
        "D&A" => Ok(0b0000000 << 6),
        "D|A" => Ok(0b0010101 << 6),
        "M" => Ok(0b1110000 << 6),
        "!M" => Ok(0b1110001 << 6),
        "-M" => Ok(0b1110011 << 6),
        "M+1" => Ok(0b1110111 << 6),
        "M-1" => Ok(0b1110010 << 6),
        "D+M" => Ok(0b1000010 << 6),
        "D-M" => Ok(0b1010011 << 6),
        "M-D" => Ok(0b1000111 << 6),
        "D&M" => Ok(0b1000000 << 6),
        "D|M" => Ok(0b1010101 << 6),
        _ => Err(Error::new(ErrorKind::InvalidSyntax)),
    }
}

/// Translates the 'jump' mnemonic in a C-command into its 16-bit binary
/// representation.
///
/// # Examples
/// "
/// '''
/// assert_eq!(jump("AM").unwrap(), 0b0000000000000101);
/// '''
pub fn jump(mnemonic: &str) -> Result<u16> {
    match mnemonic {
        "null" => Ok(0b000),
        "JGT" => Ok(0b001),
        "JEQ" => Ok(0b010),
        "JGE" => Ok(0b011),
        "JLT" => Ok(0b100),
        "JNE" => Ok(0b101),
        "JLE" => Ok(0b110),
        "JMP" => Ok(0b111),
        _ => Err(Error::new(ErrorKind::InvalidSyntax)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_dest() {
        assert_eq!(dest("null").unwrap(), 0b000000);
        assert_eq!(dest("M").unwrap(), 0b001000);
        assert_eq!(dest("D").unwrap(), 0b010000);
        assert_eq!(dest("MD").unwrap(), 0b011000);
        assert_eq!(dest("A").unwrap(), 0b100000);
        assert_eq!(dest("AM").unwrap(), 0b101000);
        assert_eq!(dest("AD").unwrap(), 0b110000);
        assert_eq!(dest("AMD").unwrap(), 0b111000);
    }

    #[test]
    #[should_panic]
    fn dest_syntax_error() {
        dest("AERTGwed").unwrap();
    }

    #[test]
    fn check_comp() {
        assert_eq!(comp("0").unwrap(), 0b0101010000000);
        assert_eq!(comp("1").unwrap(), 0b0111111000000);
        assert_eq!(comp("-1").unwrap(), 0b0111010000000);
        assert_eq!(comp("D").unwrap(), 0b0001100000000);
        assert_eq!(comp("A").unwrap(), 0b0110000000000);
        assert_eq!(comp("!D").unwrap(), 0b0001101000000);
        assert_eq!(comp("!A").unwrap(), 0b0110001000000);
        assert_eq!(comp("-D").unwrap(), 0b0001111000000);
        assert_eq!(comp("-A").unwrap(), 0b0110011000000);
        assert_eq!(comp("D+1").unwrap(), 0b0011111000000);
        assert_eq!(comp("A+1").unwrap(), 0b0110111000000);
        assert_eq!(comp("D-1").unwrap(), 0b0001110000000);
        assert_eq!(comp("A-1").unwrap(), 0b0110010000000);
        assert_eq!(comp("D+A").unwrap(), 0b0000010000000);
        assert_eq!(comp("D-A").unwrap(), 0b0010011000000);
        assert_eq!(comp("A-D").unwrap(), 0b0000111000000);
        assert_eq!(comp("D&A").unwrap(), 0b0000000000000);
        assert_eq!(comp("D|A").unwrap(), 0b0010101000000);
        assert_eq!(comp("M").unwrap(), 0b1110000000000);
        assert_eq!(comp("!M").unwrap(), 0b1110001000000);
        assert_eq!(comp("-M").unwrap(), 0b1110011000000);
        assert_eq!(comp("M+1").unwrap(), 0b1110111000000);
        assert_eq!(comp("M-1").unwrap(), 0b1110010000000);
        assert_eq!(comp("D+M").unwrap(), 0b1000010000000);
        assert_eq!(comp("D-M").unwrap(), 0b1010011000000);
        assert_eq!(comp("M-D").unwrap(), 0b1000111000000);
        assert_eq!(comp("D&M").unwrap(), 0b1000000000000);
        assert_eq!(comp("D|M").unwrap(), 0b1010101000000);
    }

    #[test]
    #[should_panic]
    fn comp_syntax_error() {
        comp("AERTGwed").unwrap();
    }

    #[test]
    fn check_jump() {
        assert_eq!(jump("null").unwrap(), 0b000);
        assert_eq!(jump("JGT").unwrap(), 0b001);
        assert_eq!(jump("JEQ").unwrap(), 0b010);
        assert_eq!(jump("JGE").unwrap(), 0b011);
        assert_eq!(jump("JLT").unwrap(), 0b100);
        assert_eq!(jump("JNE").unwrap(), 0b101);
        assert_eq!(jump("JLE").unwrap(), 0b110);
        assert_eq!(jump("JMP").unwrap(), 0b111);
    }

    #[test]
    #[should_panic]
    fn jump_syntax_error() {
        jump("AERTGwed").unwrap();
    }
}
