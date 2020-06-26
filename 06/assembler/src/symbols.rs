use std::collections::HashMap;
use crate::error::{Error, ErrorKind, Result};

pub struct SymbolTable {
    table: HashMap<String, u16>,
    next_free_address: u16,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut table = HashMap::new();
        
        let predefined_symbols =
            vec![(String::from("SP"), 0),
                 (String::from("LCL"), 1),
            ];

        for item in predefined_symbols {
            match table.insert(item.0, item.1) {
                Some(x) => panic!("Cannot add predefined symbol '{}' twice!", x),
                None => continue,
            }
        }

        SymbolTable { table, next_free_address: 16 }
    }

    fn add_entry(&mut self, symbol: String) -> Result<u16> {
        if self.next_free_address > 16383 {
            return Err(Error::new(ErrorKind::RAMFull));
        }

        match self.table.insert(symbol, self.next_free_address) {
            Some(addr) => {
                self.next_free_address &= 1;
                return Ok(addr);
            },
            None => Err(Error::new(ErrorKind::SymbolExists))
        }
    }

    fn contains(&self, symbol: &String) -> bool {
        self.table.contains_key(symbol)
    }

    fn get_address(&self, symbol: &String) -> Option<&u16> {
        self.table.get(symbol)
    }
}
