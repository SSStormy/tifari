extern crate rusqlite;
extern crate walkdir;

#[derive(Debug)]
pub enum BackendError 
{
    SQLite(rusqlite::Error),
    IO(std::io::Error),
}

impl From<std::io::Error> for BackendError 
{
    fn from(error: std::io::Error) -> Self { BackendError::IO(error) }
}

impl From<rusqlite::Error> for BackendError 
{
    fn from(error: rusqlite::Error) -> Self { BackendError::SQLite(error) }
}

pub struct TifariConfig 
{
    db_path: String,
    image_root: String,
}

impl TifariConfig 
{
    pub fn new(db_path: String, image_root: String) -> TifariConfig {
        TifariConfig { db_path, image_root }
    }
}

pub struct TifariBackend 
{
    config: TifariConfig,
    db: rusqlite::Connection,
    scan_thread: Option<std::thread::JoinHandle<()>>,
    scan_thread_comms: std::sync::mpsc::Sender<ImageThreadMessage>,
}

enum ImageThreadMessage 
{
    Quit()
}

fn scan_thread_main(path: String, receiver: std::sync::mpsc::Receiver<ImageThreadMessage>)
{
    use walkdir::WalkDir;

    for entry in WalkDir::new(path).follow_links(true) 
    {
        let entry = match entry 
        {
            Ok(entry) => entry,
            Err(err) => 
            {
                println!("[scan_thread_init] Failed to recursively acquire entry. Error: {:?}", err);
                continue;
            }
        };

        println!("{}", entry.path().display());
    }

    loop 
    {
        match receiver.recv() 
        {
            Ok(val) => match val 
            {
                ImageThreadMessage::Quit() => break,
            }
            Err(e) => println!("[scan_thread_loop] Failed to receive message. Error: {:?}", e),
        };
    }
}

impl Drop for TifariBackend 
{
    fn drop(&mut self) 
    {
        self.scan_thread_comms.send(ImageThreadMessage::Quit());
        if let Some(scan_thread) = self.scan_thread.take()
        {
            scan_thread.join();
        }
    }
}

impl TifariBackend 
{
    pub fn new(config: TifariConfig) -> Result<Self, BackendError> 
    {
        let db = rusqlite::Connection::open(config.db_path.clone())?;

        db.execute_batch("
            BEGIN;

            CREATE TABLE IF NOT EXISTS tags_by_name (
                    name TEXT PRIMARY KEY NOT NULL, 
                    id INTEGER NOT NULL);

            CREATE TABLE IF NOT EXISTS tags (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    name TEXT NOT NULL,
                    image_ids_array_table TEXT NOT NULL);

            CREATE TABLE IF NOT EXISTS images (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    path TEXT NOT NULL,
                    tags_array_table TEXT NOT NULL);

            COMMIT;
        ")?;

        std::fs::create_dir_all(config.image_root.clone())?;
        let (sender, receiver) = std::sync::mpsc::channel();

        let path = config.image_root.clone();
        let scan_thread = std::thread::spawn(move || { 
            scan_thread_main(path, receiver); 
        });

        Ok(TifariBackend { config, db, scan_thread: Some(scan_thread), scan_thread_comms: sender })
    }
}

/*
#[cfg(test)]
mod tests {
    #[test]
    pub fn 
}
*/
