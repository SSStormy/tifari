#![feature(try_trait)]
extern crate rusqlite;
extern crate notify;
extern crate walkdir;
extern crate chrono;
extern crate models;

mod error;

pub use self::error::*;

use std::collections::HashSet;

#[derive(Clone)]
pub enum DbOpenType
{
    FromPath(String),
    InMemory
}

#[derive(Clone)]
pub struct TifariConfig 
{
    db_type: DbOpenType,
    image_root: String,
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

pub struct TifariDb
{
    connection: rusqlite::Connection,
}

enum ImageThreadMessage 
{
    Quit,
}

#[derive(Debug)]
enum TagThreadMessage
{
    Rename(String, String),
    TryAdd(String),
    TryRemove(String),
    Quit,
}

impl TifariConfig 
{
    pub fn new(db_type: DbOpenType, image_root: String) -> TifariConfig
    {
        TifariConfig { db_type, image_root }
    }
}

impl TifariDb 
{
    pub fn get_image_from_db(&self, id: i64) -> Result<models::Image>
    {
        let (id, path, time, tag_array_id): (i64, String, i64, i64) = self.connection.query_row(
            "SELECT id, path, created_at_time, tags_array_table
            FROM images 
            WHERE id=?",
            &[&id],
            |row| (row.get(0), row.get(1), row.get(2), row.get(3)))?;

        let mut statement = self.connection.prepare(
            &format!("SELECT id, name 
                     FROM tags
                     WHERE id IN (SELECT tag_id from tags_array_table_{})", tag_array_id))?;

        let mut tags = HashSet::new();

        for result in statement.query_map(&[], |row| (row.get(0), row.get(1)))?
        {
            let (tag_id, tag_name) = result?;
            tags.insert(models::Tag::new(tag_id, tag_name));
        }

        Ok(models::Image::new(id, path, time, tags))
    }


    pub fn rename_image(&mut self, from: &String, to: &String) -> Result<()>
    {
        let tx = self.connection.transaction()?;

        let changed = tx.execute_named(
            "UPDATE images SET path=:to WHERE path=:from",
            &[(":from", from),
              (":to", to)])?;

        if 0 >= changed
        {
            return Err(BackendError::NoChangesOccured)
        }

        tx.commit()?;
        Ok(())
    }

    pub fn try_insert_image(&mut self, path: &String) -> Result<()>
    {
        let tx = self.connection.transaction()?;

        let exists = 
        {
            let mut statement = tx.prepare(
                "SELECT id FROM images WHERE path=? LIMIT 1")?;

            match statement.exists(&[path]) {
                Ok(val) => Ok(val),
                Err(e) => Err(BackendError::SQLite(e)),
            }
        }?;

        if exists
        {
            return Err(BackendError::ImageExists);
        }

        tx.execute_named(
            "INSERT INTO images (id, path, tags_array_table, created_at_time) 
            VALUES (null, :path, :tags_array_table, :time)",
            &[(":path", path),
              (":tags_array_table", &rusqlite::types::Null),
              (":time", &chrono::Utc::now().timestamp())]).unwrap();

        let image_id = tx.last_insert_rowid();

        tx.execute(
            &format!("CREATE TABLE IF NOT EXISTS tags_array_table_{} (tag_id INTEGER NOT NULL, UNIQUE(tag_id))", image_id), &[]).unwrap();

        let tag_table_id = tx.last_insert_rowid();

        tx.execute_named(
            "UPDATE images SET tags_array_table=:tags_array_table WHERE id=:id",
            &[(":tags_array_table", &tag_table_id),
              (":id", &image_id)]).unwrap();

        tx.execute_named(
            "INSERT INTO tag_queue (id, image_id) VALUES (null, :image_id)",
            &[(":image_id", &image_id)]).unwrap();

        tx.commit()?;
        Ok(())
    }

    fn erase_tag_if_not_used(
                             tx: &rusqlite::Transaction, 
                             tag_id: i64, image_ids_array_id: i64) -> Result<()>
    {
        // query how many elements the image ids array has.
        let num_rows_in_image_ids_table: i64 = tx.query_row(
            &format!("SELECT COUNT(*) FROM image_ids_array_table_{}", image_ids_array_id),
            &[],
            |row| row.get(0))?;

        // check if any images still reference this tag
        if 0 >= num_rows_in_image_ids_table 
        {
            // if not, drop the image_id table and the tag entry from the database
            tx.execute(
                &format!("DROP TABLE IF EXISTS image_ids_array_table_{}", image_ids_array_id),
                &[])?;

            tx.execute(
                "DELETE FROM tags WHERE id=?",
                &[&tag_id])?;
        }

        Ok(())
    }

    fn remove_image_from_tag_queue(
                                   tx: &rusqlite::Transaction, 
                                   image_id: i64) -> Result<()>
    {
        tx.execute(
            "DELETE FROM tag_queue WHERE image_id=?",
            &[&image_id])?;

        Ok(())
    }

    pub fn erase_image(&mut self, path: &String) -> Result<()>
    {
        let tx = self.connection.transaction()?;

        // get image id and it's tag array table id from db
        let (image_id, tag_array_table_id): (i64, i64) = tx.query_row(
            "SELECT id, tags_array_table FROM images WHERE path=? LIMIT 1",
            &[path],
            |row| { (row.get(0), row.get(1)) })?;

        // delete the image
        tx.execute(
            "DELETE FROM images WHERE id=?",
            &[&image_id])?;

        TifariDb::remove_image_from_tag_queue(&tx, image_id)?;
    
        // gets all the tag ids and their image id array tables that contain this image id.
        {
            let mut statement = tx.prepare(
                &format!("SELECT id, image_ids_array_table 
                         FROM tags 
                         WHERE id IN (
                             SELECT * from tags_array_table_{}
                         )", tag_array_table_id))?;

            // iterates over all the image id array table ids
            for result in statement.query_map(&[], 
                    |row| (row.get::<i32, i64>(0), 
                           row.get::<i32, i64>(1)))?
            {
                let result = result?;
                let (tag_id, image_ids_array_table_id) = (result.0, result.1);

                // ...and deletes the image id from them.
                tx.execute(
                    &format!("DELETE FROM image_ids_array_table_{} WHERE image_id=?", image_ids_array_table_id),
                    &[&image_id])?;

                // ...and erase the tag from the db if it's no longer referenced by and image
                TifariDb::erase_tag_if_not_used(&tx, tag_id, image_ids_array_table_id)?;
            }
        }

        // lastly, we drop the image's tag_array_table
        tx.execute(
            &format!("DROP TABLE IF EXISTS tags_array_table_{}", tag_array_table_id),
            &[])?;

        tx.commit()?;
        Ok(())
    }

    pub fn new_from_cfg(config: TifariConfig) -> Result<Self> 
    {
        TifariDb::new(config.db_type.clone())
    }

    pub fn new(db_type: DbOpenType) -> Result<Self>
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
                        image_ids_array_table INTEGER,
                        UNIQUE(id, name, image_ids_array_table));

                CREATE TABLE IF NOT EXISTS images (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        path TEXT NOT NULL,
                        tags_array_table INTEGER,
                        created_at_time INTEGER NOT NULL,
                        UNIQUE(id, path, tags_array_table));

                CREATE TABLE IF NOT EXISTS tag_queue (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        image_id INTEGER NOT NULL,
                        UNIQUE(id, image_id));

                COMMIT;
            ")?;

        Ok(db)
    }

    pub fn give_tag(&mut self, image_path: &String, tag: &String) -> Result<()>
    {
        let tx = self.connection.transaction()?;

        let (image_id, tags_array_table_id): (i64, i64) = tx.query_row(
            "SELECT id, tags_array_table FROM images WHERE path=? LIMIT 1",
            &[image_path],
            |row| (row.get(0), row.get(1)))?;

        let (tag_id, image_ids_array_id): (i64, i64) = match tx.query_row(
                            "SELECT id, image_ids_array_table 
                            FROM tags 
                            WHERE name=? LIMIT 1", 
                            &[tag],
                            |row| (row.get(0), row.get(1)))
        {
            Ok(tuple) => Ok(tuple),
            Err(e) =>
            {
                if let rusqlite::Error::QueryReturnedNoRows = e
                {
                    tx.execute(
                        "INSERT INTO tags (id, name, image_ids_array_table) 
                        VALUES (null, ?, null)",
                        &[tag])?;

                    let tag_id = tx.last_insert_rowid();

                    tx.execute(
                        &format!("CREATE TABLE IF NOT EXISTS image_ids_array_table_{} 
                                 (image_id INTEGER NOT NULL UNIQUE)", tag_id),
                        &[])?;

                    let table_id = tx.last_insert_rowid();

                    tx.execute_named(
                        "UPDATE tags 
                        SET image_ids_array_table=:image_ids_array_table_id
                        WHERE id=:id",
                        &[(":image_ids_array_table_id", &table_id),
                          (":id", &tag_id)])?;

                    Ok((tag_id, table_id))
                }
                else { Err(e) }
            }
        }?;

        TifariDb::remove_image_from_tag_queue(&tx, image_id)?;

        tx.execute(
            &format!("INSERT INTO image_ids_array_table_{} (image_id)
                     VALUES (?)", image_ids_array_id),
            &[&image_id])?;

        tx.execute(
            &format!("INSERT INTO tags_array_table_{} (tag_id)
                     VALUES (?)", tags_array_table_id),
            &[&tag_id])?;

        tx.commit()?;
        Ok(())
    }

    pub fn remove_tag(&mut self, image_path: &String, tag: &String) -> Result<()>
    {
        let tx = self.connection.transaction()?;

        // get tag id and image id array from db
        let (tag_id, image_ids_array_id): (i64, i64)  = tx.query_row(
            "SELECT id, image_ids_array_table FROM tags WHERE name=? LIMIT 1", 
            &[tag],
            |row| (row.get(0), row.get(1)))?;

        // get image id and tag id array fcrom db
        let (image_id, tags_array_table_id): (i64, i64) = tx.query_row(
            "SELECT id, tags_array_table FROM images WHERE path=? LIMIT 1",
            &[image_path],
            |row| (row.get(0), row.get(1)))?;
        
        // remove tag id from image tag array
        tx.execute(
            &format!("DELETE FROM tags_array_table_{} WHERE tag_id=?", tags_array_table_id),
            &[&tag_id])?;

        // remove image id from tag image array
        tx.execute(
            &format!("DELETE FROM image_ids_array_table_{} WHERE image_id=?", image_ids_array_id),
            &[&image_id])?;

        TifariDb::erase_tag_if_not_used(&tx, tag_id, image_ids_array_id)?;

        tx.commit()?;
        Ok(())
    }

    pub fn search(&self, 
                  tags: &Vec<String>, 
                  offset: usize, max_results: usize) -> Result<Vec<models::Image>>
    {
        if 0 >= tags.len()
        {
            return Ok(vec![]);
        }

        let mut query = "SELECT id FROM tags WHERE name IN (".to_string();
        let mut params: Vec<&rusqlite::types::ToSql> = Vec::with_capacity(tags.len());
        
        query.push_str(&"?, ".repeat(tags.len() - 1));
        query.push_str("?)");

        for i in 0..tags.len()
        {
            params.push(&tags[i]);
        }

        let mut statement = self.connection.prepare(&query)?;

        let mut tag_ids_query = String::from("(");
        let mut num_tags_in_query = 0;
        let mut skip_first_comma = true;
        for result in statement.query_map(params.as_slice(), |row| row.get(0))?
        {
            let result: i64 = result?;

            if !skip_first_comma
            {
                tag_ids_query.push_str(", ");
            }

            skip_first_comma = false;
            tag_ids_query.push_str(&result.to_string());
            num_tags_in_query += 1;
        }

        if num_tags_in_query == 0
        {
            return Ok(vec![]);
        }

        tag_ids_query.push(')');

        let mut statement = self.connection.prepare(
            "SELECT id, tags_array_table 
            FROM images 
            ORDER BY id DESC")?;

        let mut results = vec![];
        let mut skipped = 0;
        for result in statement.query_map(&[], |row| (row.get(0), row.get(1)))?
        {
            let (image_id, tag_array_id): (i64, i64) = result?;

            let count: i64 = self.connection.query_row(
                &format!("SELECT COUNT(*) 
                         FROM tags_array_table_{}
                         WHERE tag_id IN {}", 
                         tag_array_id, tag_ids_query),
                         &[],
                         |row| row.get(0))?;

            if count == num_tags_in_query
            {
                if skipped >= offset
                {
                    results.push(self.get_image_from_db(image_id)?);
                    if results.len() >= max_results
                    {
                        break;
                    }
                }
                else
                {
                    skipped += 1;
                }
            }
        }

        Ok(results)
    }
}

impl TifariBackend 
{
    pub fn new(config: TifariConfig) -> Result<Self> 
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

fn tag_thread_main(db_open: DbOpenType,
                   receiver: std::sync::mpsc::Receiver<TagThreadMessage>) 
{
    let mut db = TifariDb::new(db_open).unwrap();


    for recv in receiver.iter()
    {
        match recv
        {
            TagThreadMessage::TryAdd(path) =>
            {
                if let Err(e) = db.try_insert_image(&path)
                {
                    println!("[tag_thread] Error in TryAdd: {:?}", e);
                }
            }
            TagThreadMessage::TryRemove(path) =>
            {
                if let Err(e) = db.erase_image(&path)
                {
                    println!("[tag_thread] Error in TryRemove: {:?}", e);
                }
            }
            TagThreadMessage::Rename(from, to) =>
            {
                if let Err(e) = db.rename_image(&from, &to)
                {
                    println!("[tag_thread] Error in Rename: {:?}", e);
                }
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

        tag_producer.send(TagThreadMessage::TryAdd(entry.path().to_str().unwrap().to_string())).unwrap();
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
                Some(msg) => { tag_producer.send(msg).unwrap(); },
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
        self.scan_thread_comms.send(ImageThreadMessage::Quit).unwrap();
        if let Some(scan_thread) = self.scan_thread.take() { scan_thread.join().unwrap(); }


        self.tag_thread_comms.send(TagThreadMessage::Quit).unwrap();
        if let Some(tag_thread) = self.tag_thread.take() { tag_thread.join().unwrap(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_image_insertion()
    {
        let mut db = TifariDb::new(DbOpenType::InMemory).unwrap();
        db.try_insert_image(&"test/img.png".to_string()).unwrap();
        assert!(db.try_insert_image(&"test/img.png".to_string()) .is_err());
    }

    #[test]
    fn db_image_erase()
    {
        let mut db = TifariDb::new(DbOpenType::InMemory).unwrap();
        let img = "test/img.png".to_string();

        assert!(db.erase_image(&img).is_err());

        db.try_insert_image(&img).unwrap();
        db.erase_image(&img).unwrap();
    }

    #[test]
    fn db_image_rename()
    {
        let mut db = TifariDb::new(DbOpenType::InMemory).unwrap();
        let from = "test/img.png".to_string();
        let to = "test/img2.png".to_string();

        assert!(db.rename_image(&from, &to).is_err());

        db.try_insert_image(&from).unwrap();
        db.rename_image(&from, &to).unwrap();

        assert!(db.erase_image(&from).is_err());

        db.erase_image(&to).unwrap();
    }

    #[test]
    fn db_image_tag()
    {
        let mut db = TifariDb::new(DbOpenType::InMemory).unwrap();
        let img = "test/img.png".to_string();

        let tag1 = "tag_1".to_string();
        let tag2 = "tag_2".to_string();

        assert!(db.give_tag(&img, &tag1).is_err());
        assert!(db.remove_tag(&img, &tag1).is_err());
        assert!(db.give_tag(&img, &tag2).is_err());
        assert!(db.remove_tag(&img, &tag2).is_err());

        db.try_insert_image(&img).unwrap();
        db.give_tag(&img, &tag1).unwrap();

        assert!(db.give_tag(&img, &tag1).is_err());
        assert!(db.remove_tag(&img, &tag2).is_err());

        db.remove_tag(&img, &tag1).unwrap();

        assert!(db.remove_tag(&img, &tag1).is_err());

        db.give_tag(&img, &tag2).unwrap();
        assert!(db.remove_tag(&img, &tag1).is_err());
        db.remove_tag(&img, &tag2).unwrap()
    }

    #[test]
    fn db_duplicate_tags()
    {
        let mut db = TifariDb::new(DbOpenType::InMemory).unwrap();
        let img = "test/img.png".to_string();

        let tag1 = "tag_1".to_string();

        db.try_insert_image(&img).unwrap();
        db.give_tag(&img, &tag1).unwrap();

        assert!(db.give_tag(&img, &tag1).is_err());
        assert!(db.give_tag(&img, &tag1).is_err());
        assert!(db.give_tag(&img, &tag1).is_err());
    }

    #[test]
    fn db_search()
    {
        let mut db = TifariDb::new(DbOpenType::InMemory).unwrap();
        let img1 = "test/img.png".to_string();
        let img2 = "test/img2.png".to_string();
        let img3 = "test/img3.png".to_string();

        let tag1 = "tag1".to_string();
        let tag2 = "tag2".to_string();
        let tag3 = "tag3".to_string();

        db.try_insert_image(&img1).unwrap();
        db.give_tag(&img1, &tag1).unwrap();

        db.try_insert_image(&img2).unwrap();
        db.give_tag(&img2, &tag2).unwrap();

        {
            let results = db.search(&vec![&tag1], 0, 50).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].path, img1);
        }

        {
            let results = db.search(&vec![&tag2], 0, 50).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].path, img2);
        }

        db.try_insert_image(&img3).unwrap();
        db.give_tag(&img1, &tag2).unwrap();
        db.give_tag(&img3, &tag3).unwrap();

        {
            let results = db.search(&vec![&tag2], 0, 50).unwrap();
            assert_eq!(results.len(), 2);

            assert_eq!(results[0].path, img2);
            assert_eq!(results[1].path, img1);
        }

        db.give_tag(&img3, &tag1).unwrap();

        {
            let results = db.search(&vec![&tag1], 0, 50).unwrap();
            assert_eq!(results.len(), 2);

            assert_eq!(results[0].path, img3);
            assert_eq!(results[1].path, img1);
        }

        {
            let tag4 = "tag4".to_string();

            for i in 0..10
            {
                let img = format!("test/img_iter{}.png", i);
                db.try_insert_image(&img).unwrap();
                db.give_tag(&img, &tag4);
            }

            let after_10 = "special_img10".to_string();
            db.try_insert_image(&after_10).unwrap();
            db.give_tag(&after_10, &tag4);

            for i in 10..15
            {
                let img = format!("test/img_iter{}.png", i);
                db.try_insert_image(&img).unwrap();
                db.give_tag(&img, &tag4);
            }

            let after_15 = "special_img15".to_string();
            db.try_insert_image(&after_15).unwrap();
            db.give_tag(&after_15, &tag4);

            for i in 15..20
            {
                let img = format!("test/img_iter{}.png", i);
                db.try_insert_image(&img).unwrap();
                db.give_tag(&img, &tag4);
            }

            {
                let results = db.search(&vec![&tag4], 2, 10).unwrap();
                assert_eq!(results.len(), 10);

                assert!(results[0].path != after_15); // 18
                assert!(results[1].path != after_15); // 17
                assert!(results[2].path != after_15); // 16
                assert_eq!(results[3].path, after_15); // our guy
                assert!(results[4].path != after_15); // 15
            }

            {
                let results = db.search(&vec![&tag4], 4, 10).unwrap();
                assert_eq!(results.len(), 10);

                assert!(results[0].path != after_15); // 16
                assert_eq!(results[1].path, after_15); // our guy
                assert!(results[2].path != after_15); // 15

                assert!(results[6].path != after_10); // 11
                assert_eq!(results[7].path, after_10); 
                assert!(results[8].path != after_10); // 10
            }
        }
    }
}
