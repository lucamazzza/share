// MIT License
//
// Copyright (c) 2024 Luca Mazza
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
pub mod contacts {
    use redb::{Database, Error, TableDefinition};
    const TABLE: TableDefinition<&str, &str> = TableDefinition::new("contacts");
    
    const _DB_FILE: &str = "contacts.redb";
    
    pub fn _init() -> Result<(), Error> {
        let db = Database::create(_DB_FILE)?;
        let write = db.begin_write()?;
        {
            let mut table = write.open_table(_TABLE)?;
            table.insert("self", "127.0.0.1:9193")?;
        }
        write.commit()?;
        let read = db.begin_read()?;
        let table = read.open_table(_TABLE)?;
        assert_eq!(table.get("self")?.unwrap().value(), "127.0.0.1:9193");
        Ok(())
    }

    pub fn _get_addr(name: &str) -> Result<String, Error> {
        let db = Database::open(_DB_FILE)?;
        let read = db.begin_read()?;
        let table = read.open_table(_TABLE)?;
        Ok(table.get(name)?.unwrap().value().to_string())
    }
    
    pub fn _db_contains(name: &str) -> bool {
        _get_addr(name).is_ok()
    }
    
    pub fn _insert_contact(name: &str, addr: &str) -> Result<(), Error> {
        if _db_contains(name) {
        let msg = format!("Are you sure you want to overwrite {} with {}", name, addr);
            if _confirm_msg(msg.as_str()) {
                _remove_contact(name)?;
            } else {
                return Ok(());
            }
        }
        let db = Database::open(_DB_FILE)?;
        let write = db.begin_write()?;
        {
            let mut table = write.open_table(_TABLE)?;
            table.insert(name, addr)?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn _remove_contact(name: &str) -> Result<(), Error> {
        let db = Database::open(_DB_FILE)?;
        let write = db.begin_write()?;
        {
            let mut table = write.open_table(_TABLE)?;
            table.remove(name)?;
        }
        let msg= format!("Are you sure you want to remove {}", name);
        if _confirm_msg(msg.as_str()) {
            write.commit()?;
        } else {
            write.abort().expect("Aborted transaction");
        }
        Ok(())
    }

    pub fn _confirm_msg(msg: &str) -> bool{
        println!("{} [Y/n]", msg.trim());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        while !input.to_lowercase().eq("y") && !input.to_lowercase().eq("n") {
            println!("Insert 'y' or 'n'");
        }
        input.to_lowercase().eq("y")
    }
}