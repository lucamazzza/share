pub mod comm {
    use std::error::Error;
    use std::fs::File;
    use std::io::BufReader;
    use ureq;

    pub fn _send(filename: &str, contact: &str) -> Result<(), Box<dyn Error>> {
        let f = File::open(filename)?;
        let meta = f.metadata()?;
        let bufreader = BufReader::new(f);
        let fmt = format!("https://{contact}/{filename}");
        let r = ureq::post(fmt.as_str())
            .set("Content-Length", &meta.len().to_string())
            .send(bufreader);
        println!("{:?}", r);
        Ok(())
    }
}