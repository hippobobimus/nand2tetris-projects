use std::collections::HashMap;
use crate::error::{Error, ErrorKind, Result};

#[derive(Debug)]
pub struct SymbolTable {
    table: HashMap<String, u16>,
    //next_free_address: u16,
}

impl SymbolTable {
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

        for item in predefined_symbols {
            match table.insert(item.0, item.1) {
                Some(x) => panic!("Cannot add predefined symbol '{}' twice!", x),
                None => continue,
            }
        }

        SymbolTable { table }
    }

    pub fn add_entry(&mut self, symbol: String, address: u16) -> Result<u16> {
//        if self.next_free_address > 16383 {
//            return Err(Error::new(ErrorKind::RAMFull));
//        }

        if self.contains(&symbol) {
            return Err(Error::new(ErrorKind::SymbolExists));
        } else {
            self.table.insert(symbol, address);
            return Ok(address);
        }

//        match self.table.insert(symbol, address) {
//            Some(addr) => {
//                //self.next_free_address &= 1;
//                return Ok(addr);
//            },
//            None => Err(Error::new(ErrorKind::SymbolExists))
//        }
    }

    pub fn contains(&self, symbol: &String) -> bool {
        self.table.contains_key(symbol)
    }

    pub fn get_address(&self, symbol: &String) -> Option<u16> {
        self.table.get(symbol).copied()
    }
}
