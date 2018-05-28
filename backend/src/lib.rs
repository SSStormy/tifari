#![feature(try_trait)]

extern crate rusqlite;
extern crate notify;
extern crate walkdir;

#[derive(Debug)]
pub enum BackendError 
{
    SQLite(rusqlite::Error),
    IO(std::io::Error),
    Opt(std::option::NoneError),
    Notify(notify::Error),
    ImageExists,
    NoChangesOccured,
}

impl From<std::io::Error> for BackendError 
{
    fn from(error: std::io::Error) -> Self { BackendError::IO(error) }
}

impl From<rusqlite::Error> for BackendError 
{
    fn from(error: rusqlite::Error) -> Self { BackendError::SQLite(error) }
}

impl From<std::option::NoneError> for BackendError
{
    fn from(error: std::option::NoneError) -> Self { BackendError::Opt(error) }
}

impl From<notify::Error> for BackendError
{
    fn from(error: notify::Error) -> Self { BackendError::Notify(error) }
}

#[derive(Clone)]
pub enum DbOpenType
{
    FromPath(String),
    InMemory
}

pub struct TifariConfig 
{
    db_type: DbOpenType,
    image_root: String,
}

impl TifariConfig 
{
    pub fn new(db_type: DbOpenType, image_root: String) -> TifariConfig
    {
        TifariConfig { db_type, image_root }
    }
}

pub struct TifariBackend 
{
    config: TifariConfig,
    db: TifariDb,

    scan_thread: Option<std::thread::JoinHandle<()>>,
    scan_thread_comms: std::sync::mpsc::Sender<ImageThreadMessage>,

    tag_thread: Option<std::thread::JoinHandle<()>>,
    tag_thread_comms: std::sync::mpsc::Sender<TagThreadMessage>,
}

enum ImageThreadMessage 
{
    Quit,
}

use std::path::Path;
use std::collections::HashSet;

#[derive(Debug)]
enum TagThreadMessage
{
    Rename(String, String),
    TryAdd(String),
    TryRemove(String),
    Quit,
}

struct TifariDb
{
    connection: rusqlite::Connection,
}

impl TifariDb 
{
    pub fn rename_image(&self, from: &String, to: &String) -> Result<(), BackendError>
    {
        let changed = self.connection.execute_named(
            "UPDATE images SET path=:to WHERE path=:from",
            &[(":from", from),
              (":to", to)])?;

        if 0 >= changed
        {
            return Err(BackendError::NoChangesOccured)
        }

        Ok(())
    }

    pub fn try_insert_image(&self, path: &String) -> Result<(), BackendError>
    {
        let exists = self.does_image_exist(path)?;
        if exists
        {
            return Err(BackendError::ImageExists);
        }

        self.connection.execute_named(
            "INSERT INTO images (id, path, tags_array_table) VALUES (null, :path, :tags_array_table)",
            &[(":path", path),
              (":tags_array_table", &rusqlite::types::Null)]).unwrap();

        let image_id = self.connection.last_insert_rowid();

        self.connection.execute(
            &format!("CREATE TABLE IF NOT EXISTS tags_array_table_{} (tag_id INTEGER NOT NULL, UNIQUE(tag_id))", image_id), &[]).unwrap();

        let tag_table_id = self.connection.last_insert_rowid();

        self.connection.execute_named(
            "UPDATE images SET tags_array_table=:tags_array_table WHERE id=:id",
            &[(":tags_array_table", &tag_table_id),
              (":id", &image_id)]).unwrap();

        self.connection.execute_named(
            "INSERT INTO tag_queue (id, image_id) VALUES (null, :image_id)",
            &[(":image_id", &image_id)]).unwrap();

        Ok(())
    }

    pub fn erase_image(&self, path: &String) -> Result<(), BackendError>
    {
        let (image_id, image_tag_array_id): (i64, i64) = self.connection.query_row(
            "SELECT id, tags_array_table FROM images WHERE path=? LIMIT 1",
            &[path],
            |row| { (row.get(0), row.get(1)) })?;

        self.connection.execute(
            "DELETE FROM images WHERE id=?",
            &[&image_id])?;

        self.connection.execute(
            "DELETE FROM tag_queue WHERE image_id=?",
            &[&image_id])?;

        let mut statement = 
            self.connection.prepare(
                &format!("SELECT image_ids_array_table FROM tags WHERE id=(SELECT * from tags_array_table_{})", image_tag_array_id))?;

        for array_id in statement.query_map(&[], |row| row.get::<i32, i64>(0))?
        {
            self.connection.execute(
                &format!("DELETE FROM image_ids_array_table_{} WHERE image_id=?", array_id?),
                &[&image_id])?;
        }

        Ok(())
    }

    pub fn new(db_type: DbOpenType) -> Result<Self, BackendError>
    {
        let connection = match db_type 
        {
            DbOpenType::FromPath(db_path) => rusqlite::Connection::open(db_path)?,
            DbOpenType::InMemory => rusqlite::Connection::open_in_memory()?,
        };

        let db = TifariDb { connection };

        db.connection.execute_batch("
                BEGIN;

                CREATE TABLE IF NOT EXISTS tags (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        name TEXT NOT NULL,
                        image_ids_array_table TEXT,
                        UNIQUE(id, name, image_ids_array_table));

                CREATE TABLE IF NOT EXISTS images (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        path TEXT NOT NULL,
                        tags_array_table INTEGER,
                        UNIQUE(id, path, tags_array_table));

                CREATE TABLE IF NOT EXISTS tag_queue (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        image_id INTEGER NOT NULL,
                        UNIQUE(id, image_id));

                COMMIT;
            ")?;

        Ok(db)
    }

    pub fn does_image_exist(&self, path: &String) -> Result<bool, BackendError>
    {
        let mut statement = self.connection.prepare(
            "SELECT id FROM images WHERE path=? LIMIT 1")?;

        match statement.exists(&[path]) {
            Ok(val) => Ok(val),
            Err(e) => Err(BackendError::SQLite(e)),
        }
    }
}

fn tag_thread_main(db_open: DbOpenType,
                   receiver: std::sync::mpsc::Receiver<TagThreadMessage>) 
{
    let db = TifariDb::new(db_open).unwrap();

    for recv in receiver.iter()
    {
        match recv
        {
            TagThreadMessage::TryAdd(path) =>
            {
                db.try_insert_image(&path);
            }
            TagThreadMessage::TryRemove(path) =>
            {
                db.erase_image(&path);
            }
            TagThreadMessage::Rename(from, to) =>
            {
                db.rename_image(&from, &to);
            }
            TagThreadMessage::Quit => break,
        };
    }
}

fn scan_thread_main(path: String, 
                    receiver: std::sync::mpsc::Receiver<ImageThreadMessage>,
                    tag_producer: std::sync::mpsc::Sender<TagThreadMessage>)
{
    use walkdir::WalkDir;
    use notify::Watcher;

    for entry in WalkDir::new(path.clone()).follow_links(true) 
    {
        let entry = match entry 
        {
            Ok(entry) => {
                if !entry.file_type().is_file() { continue; }
                entry
            }
            Err(err) => 
            {
                println!("[scan_thread_init] Failed to recursively acquire entry. Error: {:?}", err);
                continue;
            }
        };

        tag_producer.send(TagThreadMessage::TryAdd(entry.path().to_str().unwrap().to_string()));
    }

    let (watch_prod, watch_recv) = std::sync::mpsc::channel();

    let mut watcher = notify::watcher(watch_prod, std::time::Duration::from_millis(500)).unwrap();
    watcher.watch(path.clone(), notify::RecursiveMode::Recursive).unwrap();

    loop 
    {
        match receiver.try_recv()
        {
            Ok(val) => match val 
            {
                ImageThreadMessage::Quit => break,
            }
            Err(e) => println!("[scan_thread_loop] Failed to receive message. Error: {:?}", e),
        };

        for recv in watch_recv.try_iter()
        {
            use notify::DebouncedEvent::*;
            let msg = match recv 
            {
                NoticeWrite(path) => Some(TagThreadMessage::TryAdd(path.to_string_lossy().to_string())),
                NoticeRemove(path) => Some(TagThreadMessage::TryRemove(path.to_string_lossy().to_string())),
                Create(path) => Some(TagThreadMessage::TryAdd(path.to_string_lossy().to_string())),
                Write(path) => Some(TagThreadMessage::TryAdd(path.to_string_lossy().to_string())),
                Chmod(path) => { println!("[scan_thread] Chmod at path {:?}", path); None },
                Remove(path) => Some(TagThreadMessage::TryRemove(path.to_string_lossy().to_string())),
                Rename(from, to) => Some(TagThreadMessage::Rename(from.to_string_lossy().to_string(), to.to_string_lossy().to_string())),
                Rescan => { println!("[scan_thread] Watch rescan."); None },
                Error(error, opt_buf) => { println!("[scan_thread] Debounced event error {:?} at path {:?}", error, opt_buf); None },

            };

            match msg 
            {
                Some(msg) => { tag_producer.send(msg); },
                None => (),
            };
        }

         std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

impl Drop for TifariBackend 
{
    fn drop(&mut self) 

    {
        self.scan_thread_comms.send(ImageThreadMessage::Quit);
        if let Some(scan_thread) = self.scan_thread.take() { scan_thread.join(); }


        self.tag_thread_comms.send(TagThreadMessage::Quit);
        if let Some(tag_thread) = self.tag_thread.take() { tag_thread.join(); }
    }
}

impl TifariBackend 
{
    pub fn new(config: TifariConfig) -> Result<Self, BackendError> 
    {
        let db = TifariDb::new(config.db_type.clone())?;

        std::fs::create_dir_all(config.image_root.clone())?;
        let (scan_sender, scan_receiver) = std::sync::mpsc::channel();
        let (tag_sender, tag_receiver) = std::sync::mpsc::channel();
        let tag_sender_scan_thread = tag_sender.clone();

        let path = config.image_root.clone();
        let scan_thread = std::thread::spawn(move || { 
            scan_thread_main(path, scan_receiver, tag_sender_scan_thread); 
        });


        let db_type = config.db_type.clone();
        let tag_thread = std::thread::spawn(move || {
            tag_thread_main(db_type, tag_receiver)
        });

        Ok(TifariBackend { config, db, 
            scan_thread: Some(scan_thread), scan_thread_comms: scan_sender, 
            tag_thread: Some(tag_thread), tag_thread_comms: tag_sender
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_image_insertion()
    {
        let db = TifariDb::new(DbOpenType::InMemory).unwrap();
        db.try_insert_image(&"test/img.png".to_string()).unwrap();
        match db.try_insert_image(&"test/img.png".to_string()) 
        {
            Ok(_) => assert!(false),
            Err(e) => ()
        }
    }

    #[test]
    fn db_image_erase()
    {
        let db = TifariDb::new(DbOpenType::InMemory).unwrap();
        let img = "test/img.png".to_string();

        match db.erase_image(&img)
        {
            Ok(_) => assert!(false),
            Err(e) => ()
        }

        db.try_insert_image(&img).unwrap();
        db.erase_image(&img).unwrap();
    }

    #[test]
    fn db_image_rename()
    {
        let db = TifariDb::new(DbOpenType::InMemory).unwrap();
        let from = "test/img.png".to_string();
        let to = "test/img2.png".to_string();

        match db.rename_image(&from, &to)
        {
            Ok(_) => assert!(false),
            Err(e) => ()
        }

        db.try_insert_image(&from).unwrap();
        db.rename_image(&from, &to).unwrap();

        match db.erase_image(&from)
        {
            Ok(_) => assert!(false),
            Err(e) => ()
        }

        db.erase_image(&to).unwrap();
    }
}
