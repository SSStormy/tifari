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

use std::collections::HashSet;

#[derive(Clone, Deserialize, Serialize)]
pub struct TifariConfig 
{
    db_root : String,
    image_root: String,
}

impl TifariConfig 
{
    pub fn new(db_root: String, image_root: String) -> TifariConfig
    {
        TifariConfig { db_root, image_root }
    }

    pub fn get_root(&self) -> &String { &self.image_root }
    pub fn get_db_root(&self) -> &String{ &self.db_root }
}

pub struct TifariDb
{
    config: Option<TifariConfig>,
    connection: rusqlite::Connection,
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

    fn setup_tables(&self) -> Result<()> {
        self.connection.execute_batch("
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

        Ok(())
    }

    pub fn new(cfg: TifariConfig) -> Result<Self> 
    {
        println!("nah");

        let conn = rusqlite::Connection::open(cfg.get_db_root())?;
        let db = TifariDb { config: Some(cfg), connection: conn }; 

        db.setup_tables()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self>
    {
        let conn = rusqlite::Connection::open_in_memory()?;
        let db = TifariDb { config: None, connection: conn }; 
        db.setup_tables()?;

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

    pub fn reload_root(&self) {
        match &self.config {
            Some(cfg) => {
                let path = cfg.get_root();

                use std::io;
                use std::fs::{self, DirEntry};
                use std::path::Path;

                println!("Starting root scan at {}", path);

                // TODO : gather list of stuff that is in db and on file, operate on that.
                // confg.get_root() must be absolute
                for entry in fs::read_dir(path).unwrap()
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
                        println!("Found initial file: {:?}", path);
                    }
                }

                println!("Done.");
            }
            None => ()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_image_insertion()
    {
        let mut db = TifariDb::new_in_memory().unwrap();
        db.try_insert_image(&"test/img.png".to_string()).unwrap();
        assert!(db.try_insert_image(&"test/img.png".to_string()) .is_err());
    }

    #[test]
    fn db_image_erase()
    {
        let mut db = TifariDb::new_in_memory().unwrap();
        let img = "test/img.png".to_string();

        assert!(db.erase_image(&img).is_err());

        db.try_insert_image(&img).unwrap();
        db.erase_image(&img).unwrap();
    }

    #[test]
    fn db_image_rename()
    {
        let mut db = TifariDb::new_in_memory().unwrap();
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
        let mut db = TifariDb::new_in_memory().unwrap();
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
        let mut db = TifariDb::new_in_memory().unwrap();
        let img = "test/img.png".to_string();

        let tag1 = "tag_1".to_string();

        db.try_insert_image(&img).unwrap();
        db.give_tag(&img, &tag1).unwrap();

        assert!(db.give_tag(&img, &tag1).is_err());
        assert!(db.give_tag(&img, &tag1).is_err());
        assert!(db.give_tag(&img, &tag1).is_err());
    }

    #[test]
    fn db_tag_queue() 
    {
        let mut db = TifariDb::new_in_memory().unwrap();
        let img1 = "test/img1.png".to_string();
        let img2 = "test/img2.png".to_string();

        let tag1 = "tag_1".to_string();

        assert_eq!(db.get_tag_queue().unwrap().len(), 0);

        db.try_insert_image(&img1).unwrap();
        {
            let queue = db.get_tag_queue().unwrap();
            assert_eq!(queue.len(), 1);
            assert_eq!(queue[0].get_path(), &img1);
        }

        db.try_insert_image(&img2).unwrap();
        {
            let queue = db.get_tag_queue().unwrap();
            assert_eq!(queue.len(), 2);
            assert_eq!(queue[0].get_path(), &img2);
            assert_eq!(queue[1].get_path(), &img1);
        }

        db.give_tag(&img1, &tag1).unwrap();
        {
            let queue = db.get_tag_queue().unwrap();
            assert_eq!(queue.len(), 1);
            assert_eq!(queue[0].get_path(), &img2);
        }

        db.give_tag(&img2, &tag1).unwrap();
        assert_eq!(db.get_tag_queue().unwrap().len(), 0);
    }

    #[test]
    fn db_search()
    {
        let mut db = TifariDb::new_in_memory().unwrap();
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
            let results = db.search(&vec![tag1.clone()], 0, 50).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].get_path(), &img1);
        }

        {
            let results = db.search(&vec![tag2.clone()], 0, 50).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].get_path(), &img2);
        }

        db.try_insert_image(&img3).unwrap();
        db.give_tag(&img1, &tag2).unwrap();
        db.give_tag(&img3, &tag3).unwrap();

        {
            let results = db.search(&vec![tag2.clone()], 0, 50).unwrap();
            assert_eq!(results.len(), 2);

            assert_eq!(results[0].get_path(), &img2);
            assert_eq!(results[1].get_path(), &img1);
        }

        db.give_tag(&img3, &tag1).unwrap();

        {
            let results = db.search(&vec![tag1.clone()], 0, 50).unwrap();
            assert_eq!(results.len(), 2);

            assert_eq!(results[0].get_path(), &img3);
            assert_eq!(results[1].get_path(), &img1);
        }

        {
            let tag4 = "tag4".to_string();

            for i in 0..10
            {
                let img = format!("test/img_iter{}.png", i);
                db.try_insert_image(&img).unwrap();
                db.give_tag(&img, &tag4).unwrap();
            }

            let after_10 = "special_img10".to_string();
            db.try_insert_image(&after_10).unwrap();
            db.give_tag(&after_10, &tag4).unwrap();

            for i in 10..15
            {
                let img = format!("test/img_iter{}.png", i);
                db.try_insert_image(&img).unwrap();
                db.give_tag(&img, &tag4).unwrap();
            }

            let after_15 = "special_img15".to_string();
            db.try_insert_image(&after_15).unwrap();
            db.give_tag(&after_15, &tag4).unwrap();

            for i in 15..20
            {
                let img = format!("test/img_iter{}.png", i);
                db.try_insert_image(&img).unwrap();
                db.give_tag(&img, &tag4).unwrap();
            }

            {
                let results = db.search(&vec![tag4.clone()], 2, 10).unwrap();
                assert_eq!(results.len(), 10);

                assert_ne!(results[0].get_path(), &after_15); // 18
                assert_ne!(results[1].get_path(), &after_15); // 17
                assert_ne!(results[2].get_path(), &after_15); // 16
                assert_eq!(results[3].get_path(), &after_15); // our guy
                assert_ne!(results[4].get_path(), &after_15); // 15
            }

            {
                let results = db.search(&vec![tag4.clone()], 4, 10).unwrap();
                assert_eq!(results.len(), 10);

                assert_ne!(results[0].get_path(), &after_15); // 16
                assert_eq!(results[1].get_path(), &after_15); // our guy
                assert_ne!(results[2].get_path(), &after_15); // 15

                assert_ne!(results[6].get_path(), &after_10); // 11
                assert_eq!(results[7].get_path(), &after_10); 
                assert_ne!(results[8].get_path(), &after_10); // 10
            }
        }
    }
}
