pub mod comm {
    use std::error::Error;
    use std::fs::File;
    use std::io::BufReader;
    use ureq;

    const _TMP_PATH: &str = "/tmp/share";
    
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
    
    pub fn _accept(filename: &str) {
        let filepath = format!("{_TMP_PATH}/{filename}");
        std::fs::copy(filepath.clone(), _get_sys_download_path().as_str())
            .expect("Failed to copy file");
        std::fs::remove_file(filepath).expect("Failed to remove file");
    }
    
    pub fn _reject(filename: &str) {
        std::fs::remove_file(format!("{_TMP_PATH}/{filename}"))
            .expect("Failed to remove file");
    }
    
    pub fn _get_sys_download_path() -> String {
        if std::env::consts::OS == "windows" {
            return std::env::var("USERPROFILE").unwrap() + "\\Downloads";
        } else if std::env::consts::OS == "macos" {
            return std::env::var("HOME").unwrap() + "/Downloads";
        }
        std::env::var("HOME").unwrap() + "/downloads"
    }
    
    pub fn _check_pending_requests() -> String {
        std::fs::read_to_string(_get_sys_download_path().as_str())
            .expect("Failed to read file")
    }
}