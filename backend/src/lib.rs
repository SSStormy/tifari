#![feature(try_trait)]
extern crate rusqlite;
extern crate notify;
extern crate chrono;
extern crate models;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod error;

pub use self::error::*;
use std::sync::{Arc, RwLock};

use std::collections::{HashSet, HashMap};

#[derive(Clone, Deserialize, Serialize)]
pub struct TifariConfig 
{
    api_address: String,
    frontend_address: String,
    db_root : String,
    image_root: String,
}

pub struct ScanData {
    is_scanning: std::sync::atomic::AtomicBool,
    scan_total: std::sync::atomic::AtomicUsize,
    scan_current: std::sync::atomic::AtomicUsize,
}

impl ScanData {
    pub fn default() -> Self {
        ScanData {
            is_scanning: std::sync::atomic::AtomicBool::new(false), 
            scan_total: std::sync::atomic::AtomicUsize::new(0), 
            scan_current: std::sync::atomic::AtomicUsize::new(0), 
        }
    }

    pub fn is_scanning(&self) -> bool 
    { self.is_scanning.load(std::sync::atomic::Ordering::Acquire) }

    pub fn get_scan_total(&self) -> usize
    { self.scan_total.load(std::sync::atomic::Ordering::Acquire) }

    pub fn get_scan_current(&self) -> usize
    { self.scan_current.load(std::sync::atomic::Ordering::Acquire) }

    pub fn set_is_scanning(&self, state: bool)
    { self.is_scanning.store(state, std::sync::atomic::Ordering::Release) }

    pub fn set_scan_total(&self, total: usize)
    { self.scan_total.store(total, std::sync::atomic::Ordering::Release) }

    pub fn set_scan_current(&self, current: usize)
    { self.scan_current.store(current, std::sync::atomic::Ordering::Release) }

}

impl TifariConfig 
{
    pub fn default() -> Self {
        TifariConfig { 
            api_address: String::from("127.0.0.1:8001"),
            frontend_address: String::from("127.0.0.1:3555"),
            db_root: String::from("image_and_tag.db"),
            image_root: String::from(""),
        }
    }

    pub fn get_api_address(&self) -> &String { &self.api_address }
    pub fn get_frontend_address(&self) -> &String { &self.frontend_address }
    pub fn get_root(&self) -> &String { &self.image_root }
    pub fn get_db_root(&self) -> &String{ &self.db_root }


    pub fn update(&mut self, patch: HashMap<String, String>) {
        let mut patch = patch;

        let mut apply_patch = |var: &mut String, var_name: &str| {
            match patch.remove(var_name) {
                Some(val) => *var = val,
                None => {},
            }
        };

        apply_patch(&mut self.api_address, "api_address");
        apply_patch(&mut self.frontend_address, "frontend_address");
        apply_patch(&mut self.db_root, "db_root");
        apply_patch(&mut self.image_root, "image_root");
    }
}

pub struct TifariDb
{
    connection: rusqlite::Connection,
}

impl TifariDb 
{
    pub fn get_image_from_db(&self, id: i64) -> Result<models::Image>
    {
        let (id, path, time): (i64, String, i64) = self.connection.query_row(
            "SELECT id, path, created_at_time
            FROM images 
            WHERE id=?",
            &[&id],
            |row| (row.get(0), row.get(1), row.get(2)))?;

        let mut statement = self.connection.prepare(
            &format!("SELECT id, name 
                     FROM tags
                     WHERE id IN (SELECT tag_id from tags_array_table_{})", id))?;

        let mut tags = HashSet::new();

        for result in statement.query_map(&[], |row| (row.get(0), row.get(1)))?
        {
            let (tag_id, tag_name) = result?;
            tags.insert(models::Tag::new(tag_id, tag_name));
        }

        Ok(models::Image::new(id, path, time, tags))
    }

    fn insert_into_tag_queue(tx: &rusqlite::Transaction, image_id: i64) -> Result<()> {
        tx.execute_named(
            "INSERT INTO tag_queue (id, image_id) VALUES (null, :image_id)",
            &[(":image_id", &image_id)])?;

        Ok(())
    }

    pub fn try_insert_image(&mut self, path: &str) -> Result<i64>
    {
        let tx = self.connection.transaction()?;

        let exists = 
        {
            let mut statement = tx.prepare(
                "SELECT id FROM images WHERE path=? LIMIT 1")?;

            match statement.exists(&[&path]) {
                Ok(val) => Ok(val),
                Err(e) => Err(BackendError::SQLite(e)),
            }
        }?;

        if exists
        {
            return Err(BackendError::ImageExists);
        }

        tx.execute_named(
            "INSERT INTO images (id, path, created_at_time) 
            VALUES (null, :path, :time)",
            &[(":path", &path),
              (":time", &chrono::Utc::now().timestamp())
            ])?;

        let image_id = tx.last_insert_rowid();

        tx.execute(
            &format!("CREATE TABLE IF NOT EXISTS tags_array_table_{} (tag_id INTEGER NOT NULL UNIQUE)", image_id), &[])?;

        TifariDb::insert_into_tag_queue(&tx, image_id)?;

        tx.commit()?;
        Ok(image_id)
    }

    fn erase_tag_if_not_used(tx: &rusqlite::Transaction, 
                             tag_id: i64) -> Result<()>
    {
        // query how many elements the image ids array has.
        let num_rows_in_image_ids_table: i64 = tx.query_row(
            &format!("SELECT COUNT(*) FROM image_ids_array_table_{}", tag_id),
            &[],
            |row| row.get(0))?;

        // check if any images still reference this tag
        if 0 >= num_rows_in_image_ids_table 
        {
            // if not, drop the image_id table and the tag entry from the database
            tx.execute(
                &format!("DROP TABLE IF EXISTS image_ids_array_table_{}", tag_id),
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

    pub fn erase_image(&mut self, path: &str) -> Result<()>
    {
        let tx = self.connection.transaction()?;

        // get image id and it's tag array table id from db
        let image_id: i64 = tx.query_row(
            "SELECT id FROM images WHERE path=? LIMIT 1",
            &[&path],
            |row| { row.get(0) })?;

        // delete the image
        tx.execute(
            "DELETE FROM images WHERE id=?",
            &[&image_id])?;

        TifariDb::remove_image_from_tag_queue(&tx, image_id)?;
    
        // gets all the tag ids and their image id array tables that contain this image id.
        {
            let mut statement = tx.prepare(
                &format!("SELECT id 
                         FROM tags 
                         WHERE id IN (
                             SELECT * from tags_array_table_{}
                         )", image_id))?;

            // iterates over all the image id array table ids
            for result in statement.query_map(&[], 
                    |row| row.get::<i32, i64>(0))?
            {
                let tag_id = result?;

                // ...and deletes the image id from them.
                tx.execute(
                    &format!("DELETE FROM image_ids_array_table_{} WHERE image_id=?", tag_id),
                    &[&image_id])?;

                // ...and erase the tag from the db if it's no longer referenced by and image
                TifariDb::erase_tag_if_not_used(&tx, tag_id)?;
            }
        }

        // lastly, we drop the image's tag_array_table
        tx.execute(
            &format!("DROP TABLE IF EXISTS tags_array_table_{}", image_id),
            &[])?;

        tx.commit()?;
        Ok(())
    }

    pub fn setup_tables(&self) -> Result<()> {
        self.connection.execute_batch("
            BEGIN;

            CREATE TABLE IF NOT EXISTS tags (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    name TEXT NOT NULL,
                    UNIQUE(id, name));

            CREATE TABLE IF NOT EXISTS images (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    path TEXT NOT NULL,
                    created_at_time INTEGER NOT NULL,
                    UNIQUE(id, path));

            CREATE TABLE IF NOT EXISTS tag_queue (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    image_id INTEGER NOT NULL,
                    UNIQUE(id, image_id));

            COMMIT;
        ")?;

        Ok(())
    }

    pub fn new(cfg: Arc<RwLock<TifariConfig>>) -> Result<Self> 
    {
        let conn = rusqlite::Connection::open(cfg.read().unwrap().get_db_root())?;
        let db = TifariDb { connection: conn };

        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self>
    {
        let conn = rusqlite::Connection::open_in_memory()?;
        let db = TifariDb { connection: conn };

        Ok(db)
    }

    pub fn give_tag(&mut self, image_id: i64, tag: &str) -> Result<i64>
    {
        if tag.starts_with("-") || 
            tag.starts_with(" ") ||
            tag.starts_with("\t") ||
            tag.starts_with("\r") ||
            tag.starts_with("\n") ||
            tag.len() <= 0 {
            return Err(BackendError::BadTag); 
        }

        let tx = self.connection.transaction()?;

        let tag_id: i64 = match tx.query_row(
                            "SELECT id
                            FROM tags 
                            WHERE name=? LIMIT 1", 
                            &[&tag],
                            |row| row.get(0))
        {
            Ok(tuple) => Ok(tuple),
            Err(e) =>
            {
                if let rusqlite::Error::QueryReturnedNoRows = e
                {
                    tx.execute(
                        "INSERT INTO tags (id, name)
                        VALUES (null, ?)",
                        &[&tag])?;

                    let tag_id = tx.last_insert_rowid();

                    tx.execute(
                        &format!("CREATE TABLE IF NOT EXISTS image_ids_array_table_{} 
                                 (image_id INTEGER NOT NULL UNIQUE)", tag_id),
                        &[])?;

                    Ok(tag_id)
                }
                else { Err(e) }
            }
        }?;

        TifariDb::remove_image_from_tag_queue(&tx, image_id)?;

        tx.execute(
            &format!("INSERT INTO image_ids_array_table_{} (image_id)
                     VALUES (?)", tag_id),
            &[&image_id])?;

        tx.execute(
            &format!("INSERT INTO tags_array_table_{} (tag_id)
                     VALUES (?)", image_id),
            &[&tag_id])?;

        tx.commit()?;
        Ok(tag_id)
    }

    pub fn remove_tag(&mut self, image_id: i64, tag_id: i64) -> Result<()>
    {
        let tx = self.connection.transaction()?;

        // remove tag id from image tag array
        tx.execute(
            &format!("DELETE FROM tags_array_table_{} WHERE tag_id=?", image_id),
            &[&tag_id])?;

        // remove image id from tag image array
        tx.execute(
            &format!("DELETE FROM image_ids_array_table_{} WHERE image_id=?", tag_id),
            &[&image_id])?;

        TifariDb::erase_tag_if_not_used(&tx, tag_id)?;

        // get the number of tags this image has
        let tag_count: i64 = tx.query_row(
            &format!("SELECT count(*) FROM tags_array_table_{}", image_id),
            &[],
            |row| row.get(0))?;

        if 0 >= tag_count {
            TifariDb::insert_into_tag_queue(&tx, image_id)?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_tag_queue(&self) -> Result<Vec<models::Image>> 
    {
        let mut statement = self.connection.prepare(
            "SELECT id, path, created_at_time
            FROM images
            WHERE id IN (SELECT image_id 
                        FROM tag_queue)
            ORDER BY id DESC"
            )?;

        let mut results = vec![];
        for result in statement.query_map(&[], |row| (row.get(0), row.get(1), row.get(2)))?
        {
            let result = result?;
            let (id, path, created_at_time) = (result.0, result.1, result.2);

            results.push(models::Image::new_no_tags(id, path, created_at_time));
        }

        Ok(results)
    }

    fn make_tag_id_list(&self, tags: &Vec<&str>) -> Result<String>
    {
        if tags.len() <= 0
        {
            return Ok(String::from("()"));
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
        }

        tag_ids_query.push(')');

        Ok(tag_ids_query)
    }

    pub fn search(&self, tags: &Vec<&str>) -> Result<Vec<models::Image>>
    {
        if 0 >= tags.len()
        {
            return Ok(vec![]);
        }

        let mut tags_contains = vec![];
        let mut tags_remove = vec![];

        for tag in tags {
            if tag.starts_with("-") { tags_remove.push(&tag[1..]); }
            else { tags_contains.push(*tag); }
        }

        let tag_ids_query = self.make_tag_id_list(&tags_contains)?;
        let not_in_tag_ids = self.make_tag_id_list(&tags_remove)?;

        let mut statement = self.connection.prepare(
            "SELECT id 
            FROM images 
            ORDER BY id DESC")?;

        let mut results = vec![];
        for result in statement.query_map(&[], |row| row.get(0))?
        {
            let image_id: i64 = result?;

            let count: i64 = self.connection.query_row(
                &format!("SELECT COUNT(*) 
                         FROM tags_array_table_{}
                         WHERE tag_id IN {} 
                         AND NOT EXISTS(SELECT 1 FROM tags_array_table_{} WHERE tag_id IN {})",
                         image_id, tag_ids_query, image_id, not_in_tag_ids),
                         &[],
                         |row| row.get(0))?;

            let count = count as usize;

            if count == tags_contains.len()
            {
                results.push(self.get_image_from_db(image_id)?);
            }
        }

        Ok(results)
    }

    pub fn get_num_elements_in_tag_queue(&self) -> Result<i64> {

        let retval = self.connection.query_row(
            "SELECT count(*) FROM tag_queue", 
            &[],
            |row| row.get(0))?;

        Ok(retval)
    }

    pub fn get_all_tags(&self) -> Result<Vec<models::TagWithUsage>> {
        let mut statement = self.connection.prepare("SELECT id, name FROM tags")?;
        let mut retvals = vec![];

        for result in statement.query_map(&[], |row| (row.get(0), row.get(1)))? {
            let result = result?;
            let (id, name) = (result.0, result.1);

            let num_times_used = self.connection.query_row(
                &format!("SELECT count(*) FROM image_ids_array_table_{}", id), 
                &[],
                |row| row.get(0))?;

            retvals.push(models::TagWithUsage::new(id, name, num_times_used));
        }

        Ok(retvals)
    }

    fn get_all_image_paths(&self) -> Result<HashSet<String>> {
        let mut db_imgs = HashSet::new();
        let mut statement = self.connection.prepare("SELECT path FROM images")?;

        for result in statement.query_map(&[], |row| row.get(0))? {
            db_imgs.insert(result?);
        }

        Ok(db_imgs)
    }

    pub fn reload_root_unsafe(&mut self, root: &str, scan: &Arc<ScanData>) {
        use std::fs;

        println!("Starting root scan at \"{}\"", root);

        let iter = match fs::read_dir(root) {
            Ok(v) => v,
            Err(_) => { 
                println!("Failed to start root scan. The image root directory is most likely invalid.");
                return;
            }
        };


        let mut root_imgs = HashSet::new();

        for entry in iter
        {
        
            let entry = match entry {
                Ok(e) => e,
                Err(e) => { 
                    println!("Error in initial root scan: {:?}", e);
                    continue;
                }
            };

            let data = match entry.metadata() {
                Ok(e) => e,
                Err(e) => {
                    println!("Error in initial root scan: {:?}", e);
                    continue;
                }
            };

            if data.is_file() {
                let path = entry.path().file_name().unwrap().to_string_lossy().to_string();
//                println!("Found initial file: {:?}", path);
                root_imgs.insert(path);
            }
        }

        let db_imgs = match self.get_all_image_paths() {
            Ok(imgs) => imgs,
            Err(e) => {
                println!("Failed to query paths in image table from database: {:?}", e);
                return;
            }
        };

        scan.set_scan_total(db_imgs.difference(&root_imgs).count() + root_imgs.difference(&db_imgs).count());

        let mut scan_current = 1;

        for path_to_rm in db_imgs.difference(&root_imgs) {

            scan.set_scan_current(scan_current);
            scan_current += 1;

            
            match self.erase_image(&path_to_rm) {
                Ok(()) => {}, //println!("Erasing image: {}", path_to_rm),
                Err(e) => println!("failed to erase image {}. Error: {:?}", path_to_rm, e),
            } 
        }

        for path_to_add in root_imgs.difference(&db_imgs) {

            scan.set_scan_current(scan_current);
            scan_current += 1;

            match self.try_insert_image(&path_to_add) {
                Ok(id) => {},//println!("Adding new image: {} {}", path_to_add, id),
                Err(e) => println!("Failed to insert new image {} to image db. Error: {:?}", path_to_add, e),
            };
        }
        
        println!("Done.");

    }

    pub fn reload_root(&mut self, root: &str, scan: Arc<ScanData>) {
        if scan.is_scanning.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }

        scan.set_is_scanning(true);

        self.reload_root_unsafe(root, &scan);

        scan.set_is_scanning(false);
        scan.set_scan_total(0);
        scan.set_scan_current(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_image_insertion()
    {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();
        db.try_insert_image(&"test/img.png").unwrap();
        assert!(db.try_insert_image(&"test/img.png") .is_err());
    }

    #[test]
    fn db_image_erase()
    {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();
        let img = "test/img.png";

        assert!(db.erase_image(&img).is_err());

        db.try_insert_image(&img).unwrap();
        db.erase_image(&img).unwrap();
    }

    #[test]
    fn db_image_tag()
    {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();
        let img = "test/img.png";

        let tag1 = "tag_1";
        let tag2 = "tag_2";

        assert!(db.give_tag(1, &tag1).is_err());
        assert!(db.remove_tag(1, 1).is_err());
        assert!(db.give_tag(1, &tag2).is_err());
        assert!(db.remove_tag(1, 2).is_err());

        let img_id = db.try_insert_image(&img).unwrap();
        let tag1_id = db.give_tag(img_id, &tag1).unwrap();

        assert!(db.give_tag(img_id, &tag1).is_err());
        assert!(db.remove_tag(img_id, 2).is_err());

        db.remove_tag(img_id, tag1_id).unwrap();

        assert!(db.remove_tag(img_id, tag1_id).is_err());

        let tag2_id = db.give_tag(img_id, &tag2).unwrap();
        assert!(db.remove_tag(img_id, tag1_id).is_err());
        db.remove_tag(img_id, tag2_id).unwrap()
    }

    #[test]
    fn db_consistent_tag_ids() {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();

        let img1 = "img1";
        let img2 = "img2";
        let img3 = "img3";
        let img4 = "img4";

        let tag1 = "tag1";
        let tag2 = "tag2";

        let img1_id = db.try_insert_image(&img1).unwrap();
        let img2_id = db.try_insert_image(&img2).unwrap();

        let tag1_id = db.give_tag(img1_id, &tag1).unwrap();
        assert_eq!(db.give_tag(img2_id, &tag1).unwrap(), tag1_id);

        let tag2_id = db.give_tag(img1_id, &tag2).unwrap();
        assert_eq!(db.give_tag(img2_id, &tag2).unwrap(), tag2_id);

        assert_ne!(tag1_id, tag2_id);

        let img3_id = db.try_insert_image(&img3).unwrap();
        let img4_id = db.try_insert_image(&img4).unwrap();

        let tag1_id2 = db.give_tag(img3_id, &tag1).unwrap();
        assert_eq!(db.give_tag(img4_id, &tag1).unwrap(), tag1_id2);

        let tag2_id2 = db.give_tag(img3_id, &tag2).unwrap();
        assert_eq!(db.give_tag(img4_id, &tag2).unwrap(), tag2_id2);

        assert_ne!(tag1_id2, tag2_id2);

        assert_eq!(tag1_id, tag1_id2);
        assert_eq!(tag2_id, tag2_id2);
    }

    #[test]
    fn db_duplicate_tags()
    {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();
        let img = "test/img.png";

        let tag1 = "tag_1";

        let img_id = db.try_insert_image(&img).unwrap();
        db.give_tag(img_id, &tag1).unwrap();

        assert!(db.give_tag(img_id, &tag1).is_err());
        assert!(db.give_tag(img_id, &tag1).is_err());
        assert!(db.give_tag(img_id, &tag1).is_err());
    }

    #[test]
    fn db_tag_queue_element_counter() {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();

        let img1 = "test/img1.png";
        let img2 = "test/img2.png";

        let tag1 = "tag_1";

        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 0);

        db.try_insert_image(&img1).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 1);

        let img2_id = db.try_insert_image(&img2).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 2);

        db.erase_image(&img1).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 1);

        let tag1_id = db.give_tag(img2_id, &tag1).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 0);

        let img1_id = db.try_insert_image(&img1).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 1);

        db.remove_tag(img2_id, tag1_id).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 2);

        db.give_tag(img1_id, &tag1).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 1);

        db.erase_image(&img2).unwrap();
        assert_eq!(db.get_num_elements_in_tag_queue().unwrap(), 0);
    }

    #[test]
    fn db_tag_queue_gets_filled_when_tags_are_removed_from_images() {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();

        let img1 = "test/img1.png";
        let img2 = "test/img2.png";

        let tag1 = "tag_1";

        let img1_id = db.try_insert_image(&img1).unwrap();
        let img2_id = db.try_insert_image(&img2).unwrap();

        {
            let queue = db.get_tag_queue().unwrap();

            assert_eq!(queue.len(), 2);

            assert_eq!(queue[0].get_path(), &img2);
            assert_eq!(queue[0].get_id(), img2_id);

            assert_eq!(queue[1].get_path(), &img1);
            assert_eq!(queue[1].get_id(), img1_id);
        }

        let tag1_id = db.give_tag(img2_id, &tag1).unwrap();

        {
            let queue = db.get_tag_queue().unwrap();

            assert_eq!(queue.len(), 1);

            assert_eq!(queue[0].get_path(), &img1);
            assert_eq!(queue[0].get_id(), img1_id);
        }


        assert_eq!(db.give_tag(img1_id, &tag1).unwrap(), tag1_id);
        let queue = db.get_tag_queue().unwrap();
        assert_eq!(queue.len(), 0);

        db.remove_tag(img1_id, tag1_id).unwrap();

        {
            let queue = db.get_tag_queue().unwrap();

            assert_eq!(queue.len(), 1);

            assert_eq!(queue[0].get_path(), &img1);
            assert_eq!(queue[0].get_id(), img1_id);
        }


        assert_eq!(db.give_tag(img1_id, &tag1).unwrap(), tag1_id);
        let queue = db.get_tag_queue().unwrap();
        assert_eq!(queue.len(), 0);

        db.remove_tag(img1_id, tag1_id).unwrap();
        db.remove_tag(img2_id, tag1_id).unwrap();

        {
            let queue = db.get_tag_queue().unwrap();

            assert_eq!(queue.len(), 2);

            assert_eq!(queue[0].get_path(), &img2);
            assert_eq!(queue[0].get_id(), img2_id);

            assert_eq!(queue[1].get_path(), &img1);
            assert_eq!(queue[1].get_id(), img1_id);
        }
    }

    #[test]
    fn db_tag_queue() 
    {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();
        let img1 = "test/img1.png";
        let img2 = "test/img2.png";

        let tag1 = "tag_1";

        assert_eq!(db.get_tag_queue().unwrap().len(), 0);

        let img1_id = db.try_insert_image(&img1).unwrap();
        {
            let queue = db.get_tag_queue().unwrap();
            assert_eq!(queue.len(), 1);

            assert_eq!(queue[0].get_path(), &img1);
            assert_eq!(queue[0].get_id(), img1_id);
        }

        let img2_id = db.try_insert_image(&img2).unwrap();
        {
            let queue = db.get_tag_queue().unwrap();
            assert_eq!(queue.len(), 2);

            assert_eq!(queue[0].get_path(), &img2);
            assert_eq!(queue[0].get_id(), img2_id);

            assert_eq!(queue[1].get_path(), &img1);
            assert_eq!(queue[1].get_id(), img1_id);
        }

        db.give_tag(img1_id, &tag1).unwrap();
        {
            let queue = db.get_tag_queue().unwrap();
            assert_eq!(queue.len(), 1);

            assert_eq!(queue[0].get_path(), &img2);
            assert_eq!(queue[0].get_id(), img2_id);
        }

        db.give_tag(img2_id, &tag1).unwrap();
        assert_eq!(db.get_tag_queue().unwrap().len(), 0);
    }

    #[test]
    fn db_search_with_removals() {
        let img1 = "test/img.png";
        let img2 = "test/img2.png";

        let tag1 = "tag1";
        let tag2 = "tag2";
        let tag3 = "tag3";

        let no_tag3 = "-tag3";

        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();

        let img1_id = db.try_insert_image(&img1).unwrap();
        let img2_id = db.try_insert_image(&img2).unwrap();

        let tag1_id = db.give_tag(img1_id, &tag1).unwrap();
        let tag2_id = db.give_tag(img1_id, &tag2).unwrap();
        let tag3_id = db.give_tag(img1_id, &tag3).unwrap();

        assert_eq!(tag1_id, db.give_tag(img2_id, &tag1).unwrap());
        assert_eq!(tag2_id, db.give_tag(img2_id, &tag2).unwrap());

        {
            let results = db.search(&vec![&tag1, &tag2]).unwrap();
            assert_eq!(results.len(), 2);
        }

        {
            let results = db.search(&vec![&tag1, &tag2, &no_tag3]).unwrap();
            assert_eq!(results.len(), 1);

            assert_eq!(results[0].get_path(), &img2);
            assert_eq!(results[0].get_id(), img2_id);
        }
    }

    #[test]
    fn db_disallow_some_tags() {
        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();
        let img = db.try_insert_image("image").unwrap();

        assert!(db.give_tag(img, &"").is_err());
        assert!(db.give_tag(img, &"-").is_err());
        assert!(db.give_tag(img, &"-asdf").is_err());
        assert!(db.give_tag(img, &" -asdf").is_err());
        assert!(db.give_tag(img, &"\n").is_err());
        assert!(db.give_tag(img, &"\r").is_err());
        assert!(db.give_tag(img, &"\t").is_err());
    }

    #[test]
    fn db_search() {
        let img1 = "test/img.png";
        let img2 = "test/img2.png";
        let img3 = "test/img3.png";

        let tag1 = "tag1";
        let tag2 = "tag2";
        let tag3 = "tag3";

        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();

        let img1_id = db.try_insert_image(&img1).unwrap();
        let tag1_id = db.give_tag(img1_id, &tag1).unwrap();
        let tag2_id = db.give_tag(img1_id, &tag2).unwrap();
        assert_ne!(tag1_id, tag2_id);

        {
            let results = db.search(&vec![&tag3, &tag2]).unwrap();
            assert_eq!(results.len(), 0);
        }

        {
            let results = db.search(&vec![&tag1, &tag2]).unwrap();
            assert_eq!(results.len(), 1);

            assert_eq!(results[0].get_path(), &img1);
            assert_eq!(results[0].get_id(), img1_id);

        }

        {
            let results = db.search(&vec![&tag2, &tag1]).unwrap();
            assert_eq!(results.len(), 1);

            assert_eq!(results[0].get_path(), &img1);
            assert_eq!(results[0].get_id(), img1_id);
        }

        let mut db = TifariDb::new_in_memory().unwrap().setup_tables();

        let img1_id = db.try_insert_image(&img1).unwrap();
        let tag1_id = db.give_tag(img1_id, &tag1).unwrap();

        let img2_id = db.try_insert_image(&img2).unwrap();
        let tag2_id = db.give_tag(img2_id, &tag2).unwrap();

        {
            let results = db.search(&vec![&tag1]).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].get_path(), &img1);
            assert_eq!(results[0].get_id(), img1_id);
        }

        {
            let results = db.search(&vec![&tag2]).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].get_path(), &img2);
            assert_eq!(results[0].get_id(), img2_id);
        }

        let img3_id = db.try_insert_image(&img3).unwrap();
        assert_eq!(db.give_tag(img1_id, &tag2).unwrap(), tag2_id);
        db.give_tag(img3_id, &tag3).unwrap();

        {
            let results = db.search(&vec![&tag2]).unwrap();
            assert_eq!(results.len(), 2);

            assert_eq!(results[0].get_path(), &img2);
            assert_eq!(results[0].get_id(), img2_id);

            assert_eq!(results[1].get_path(), &img1);
            assert_eq!(results[1].get_id(), img1_id);
        }

        assert_eq!(db.give_tag(img3_id, &tag1).unwrap(), tag1_id);

        {
            let results = db.search(&vec![&tag1]).unwrap();
            assert_eq!(results.len(), 2);

            assert_eq!(results[0].get_path(), &img3);
            assert_eq!(results[0].get_id(), img3_id);

            assert_eq!(results[1].get_path(), &img1);
            assert_eq!(results[1].get_id(), img1_id);
        }
    }
}
