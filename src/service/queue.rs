use std::collections::HashMap;

use r2d2_sqlite::SqliteConnectionManager;

pub struct Queue<'a> {
    db_pool: &'a r2d2::Pool<SqliteConnectionManager>,
}

#[derive(Debug, Clone)]
pub struct QueueEntity {
    pub id: Option<u32>,
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub tag: (String, String),
}

impl<'a> Queue<'a> {
    pub fn new(db_pool: &'a r2d2::Pool<SqliteConnectionManager>) -> Self {
        Queue { db_pool }
    }

    /// Create queue attributes in the database
    /// If the attribute exists, update the value
    /// Attributes come from the https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_CreateQueue.html
    pub fn create_attributes(
        &self,
        queue_id: usize,
        attributes: HashMap<String, String>,
    ) -> Result<(), std::io::Error> {
        let conn = match self.db_pool.get() {
            Ok(conn) => conn,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };

        let mut stmt = conn
            .prepare(
                r#"
                INSERT INTO `attributes` (`queue_id`, `name`, `value`) 
                VALUES (?1, ?2, ?3) ON CONFLICT (`queue_id`, `name`) 
                DO UPDATE SET `value` = ?3
                "#,
            )
            .unwrap();

        for (key, value) in attributes {
            match stmt.execute(&[&queue_id.to_string(), &key, &value]) {
                Ok(_) => {}
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            }
        }

        Ok(())
    }

    /// Get queue in the database
    pub fn create_queue(&self, queue: QueueEntity) -> Result<String, std::io::Error> {
        let conn = match self.db_pool.get() {
            Ok(conn) => conn,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };

        return match conn.execute(
            "INSERT INTO `queues` (`name`, `tag_name`, `tag_value`) VALUES (?1, ?2, ?3)",
            &[&queue.name, &queue.tag.0, &queue.tag.1],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid().to_string()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };
    }
}
