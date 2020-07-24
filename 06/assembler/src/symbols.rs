use std::collections::HashMap;
use crate::error::{Error, ErrorKind, Result};

/// The SymbolTable is a hashmap that holds both label and variable symbols along with their
/// associated ROM or RAM address respectively.
///
/// It also tracks the next available ROM and RAM addresses which are used when inserting a new
/// symbol.
///
#[derive(Debug)]
pub struct SymbolTable {
    table: HashMap<String, u16>,
    ram_address: u16,
    rom_address: u16,
}

impl SymbolTable {
    /// Creates a new SymbolTable instance, initialised with a number of predefined variable
    /// symbols and their associated RAM addresses.
    ///
    /// The next avaialble RAM address is therefore set appropriately to 16, whilst the next ROM
    /// address is set to 0.
    ///
    pub fn new() -> SymbolTable {
        let mut table = HashMap::new();
        
        let predefined_symbols =
            vec![(String::from("SP"), 0),
                 (String::from("LCL"), 1),
                 (String::from("ARG"), 2),
                 (String::from("THIS"), 3),
                 (String::from("THAT"), 4),
                 (String::from("R0"), 0),
                 (String::from("R1"), 1),
                 (String::from("R2"), 2),
                 (String::from("R3"), 3),
                 (String::from("R4"), 4),
                 (String::from("R5"), 5),
                 (String::from("R6"), 6),
                 (String::from("R7"), 7),
                 (String::from("R8"), 8),
                 (String::from("R9"), 9),
                 (String::from("R10"), 10),
                 (String::from("R11"), 11),
                 (String::from("R12"), 12),
                 (String::from("R13"), 13),
                 (String::from("R14"), 14),
                 (String::from("R15"), 15),
                 (String::from("SCREEN"), 16384),
                 (String::from("KBD"), 24576),
            ];

        // Check for duplication.
        for item in predefined_symbols {
            match table.insert(item.0, item.1) {
                Some(x) => panic!("Cannot add predefined symbol '{}' twice!", x),
                None => continue,
            }
        }

        SymbolTable {
            table,
            ram_address: 16, // Next available.
            rom_address: 0,
        }
    }

    /// When provided with a symbol, references the SymbolTable and returns an option containing
    /// the associated ROM/RAM address.  Returns None if the symbol is not found in the
    /// SymbolTable.
    /// 
    pub fn get_address(&self, symbol: &str) -> Option<u16> {
        self.table.get(symbol).copied()
    }

    /// Increments the next available RAM address by 1 and returns Ok(0).
    ///
    /// If all the RAM addresses have already been used, a 'RAMFull' error will be returned.
    ///
    pub fn inc_ram_address(&mut self) -> Result<u8> {
        if self.ram_address == 16383 {
            return Err(Error::new(ErrorKind::RAMFull));
        }

        self.ram_address += 1;

        Ok(0)
    }

    /// Increment next ROM address by 1.
    ///
    pub fn inc_rom_address(&mut self) {
        self.rom_address += 1;
    }

    /// Takes an &str variable symbol as an argument and inserts it into the SymbolTable with the
    /// next available RAM address.
    ///
    /// Returns a result containing the RAM address.  It will return an error if the symbol already
    /// exists in the SymbolTable.
    ///
    pub fn insert_variable(&mut self, symbol: &str) -> Result<u16> {
        self.insert(symbol, self.ram_address)
    }

    /// Takes an &str label symbol as an argument and inserts it into the SymbolTable with the
    /// next ROM address.
    ///
    /// Returns a result containing the ROM address.  It will return an error if the symbol already
    /// exists in the SymbolTable.
    ///
    pub fn insert_label(&mut self, symbol: &str) -> Result<u16> {
        self.insert(symbol, self.rom_address)
    }

    fn insert(&mut self, symbol: &str, address: u16) -> Result<u16> {
        if self.table.contains_key(symbol) {
            return Err(Error::new(ErrorKind::SymbolExists));
        } else {
            self.table.insert(String::from(symbol), address);
            return Ok(address);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_predefined_symbols() {
        let sym_table = SymbolTable::new();

        let predefined_symbols =
            vec![(String::from("SP"), 0),
                 (String::from("LCL"), 1),
                 (String::from("ARG"), 2),
                 (String::from("THIS"), 3),
                 (String::from("THAT"), 4),
                 (String::from("R0"), 0),
                 (String::from("R1"), 1),
                 (String::from("R2"), 2),
                 (String::from("R3"), 3),
                 (String::from("R4"), 4),
                 (String::from("R5"), 5),
                 (String::from("R6"), 6),
                 (String::from("R7"), 7),
                 (String::from("R8"), 8),
                 (String::from("R9"), 9),
                 (String::from("R10"), 10),
                 (String::from("R11"), 11),
                 (String::from("R12"), 12),
                 (String::from("R13"), 13),
                 (String::from("R14"), 14),
                 (String::from("R15"), 15),
                 (String::from("SCREEN"), 16384),
                 (String::from("KBD"), 24576),
            ];

        for item in predefined_symbols {
            assert_eq!(
                sym_table.get_address(&item.0[..]).unwrap(),
                item.1,
            );
        }
    }

    #[test]
    fn inc_addresses() {
        let mut sym_table = SymbolTable::new();

        sym_table.inc_ram_address().unwrap();

        let ram_address = sym_table.insert_variable("TEST_VAR").unwrap();

        assert_eq!(ram_address, 17);

        sym_table.inc_rom_address();

        let rom_address = sym_table.insert_label("TEST_LABEL").unwrap();

        assert_eq!(rom_address, 1);
    }

    #[test]
    #[should_panic(expected = "there are no more free RAM addresses")]
    fn inc_ram_address_beyond_max() {
        let mut sym_table = SymbolTable::new();

        let start_address = 16;
        let max_address = 16383;

        for _ in start_address..(max_address + 1) {
            sym_table.inc_ram_address().unwrap();
        }
    }

    #[test]
    fn verify_insert_variable() {
        let mut sym_table = SymbolTable::new();

        let symbol = "NEW_VAR";

        let address = sym_table.insert_variable(symbol).unwrap();

        assert_eq!(address, 16);

        assert_eq!(
            sym_table.get_address(symbol).unwrap(),
            address,
        );
    }

    #[test]
    fn verify_insert_label() {
        let mut sym_table = SymbolTable::new();

        let symbol = "NEW_LOOP";

        let address = sym_table.insert_label(symbol).unwrap();

        assert_eq!(address, 0);

        assert_eq!(
            sym_table.get_address(symbol).unwrap(),
            address,
        );
    }
}
