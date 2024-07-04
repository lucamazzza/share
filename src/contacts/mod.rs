pub mod contacts {
    use redb::{Database, Error, TableDefinition};
    const TABLE: TableDefinition<&str, &str> = TableDefinition::new("contacts");
    
    pub fn _init() -> Result<(), Error> {
        let db = Database::create("contacts.redb")?;
        let write = db.begin_write()?;
        {
            let mut table = write.open_table(TABLE)?;
            table.insert("self", "127.0.0.1:9193")?;
        }
        write.commit()?;
        let read = db.begin_read()?;
        let table = read.open_table(TABLE)?;
        assert_eq!(table.get("self")?.unwrap().value(), "127.0.0.1:9193");
        Ok(())
    }

    pub fn _get_addr(name: &str) -> Result<String, Error> {
        let db = Database::open("contacts.redb")?;
        let read = db.begin_read()?;
        let table = read.open_table(TABLE)?;
        Ok(table.get(name)?.unwrap().value().to_string())
    }
    
    pub fn _insert_contact(name: &str, addr: &str) -> Result<(), Error> {
        let db = Database::open("contacts.redb")?;
        let write = db.begin_write()?;
        {
            let mut table = write.open_table(TABLE)?;
            table.insert(name, addr)?;
        }
        write.commit()?;
        Ok(())
    }

    pub fn _remove_contact(name: &str) -> Result<(), Error> {
        let db = Database::open("contacts.redb")?;
        let write = db.begin_write()?;
        {
            let mut table = write.open_table(TABLE)?;
            table.remove(name)?;
        }
        write.commit()?;
        Ok(())
    }
}