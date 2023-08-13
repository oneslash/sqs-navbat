use std::collections::HashMap;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use sqlx::SqlitePool;

pub struct Queue<'a> {
    db_pool: &'a SqlitePool,
    hostname: &'a str,
}

#[derive(Debug, Clone)]
pub struct QueueEntity {
    pub id: Option<i64>,
    pub name: String,
    pub attributes: Option<HashMap<String, String>>,

    pub tag_name: Option<String>,
    pub tag_value: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl<'a> Queue<'a> {
    pub fn new(db_pool: &'a SqlitePool, hostname: &'a str) -> Self {
        Queue { db_pool, hostname }
    }

    /// Create queue attributes in the database
    /// If the attribute exists, update the value
    /// Attributes come from the https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_CreateQueue.html
    pub async fn create_attributes(
        &self,
        queue_id: u32,
        attributes: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        for (key, value) in attributes {
            sqlx::query!(
                r#"
                INSERT INTO `attributes` (`queue_id`, `name`, `value`) 
                VALUES ($1, $2, $3) 
                "#,
                queue_id,
                key,
                value
            )
            .execute(self.db_pool)
            .await?;
        }

        Ok(())
    }

    /// Get queue in the database
    pub async fn create_queue(&self, queue: QueueEntity) -> anyhow::Result<String> {
        let inserted_id = sqlx::query!(
            r#"
            INSERT INTO `queues` (`name`, `tag_name`, `tag_value`) 
            VALUES ($1, $2, $3) 
            "#,
            queue.name,
            queue.tag_name,
            queue.tag_value
        )
        .execute(self.db_pool)
        .await?
        .last_insert_rowid();

        return Ok(inserted_id.to_string());
    }

    pub async fn list_queue(
        &self,
        max_results: u32,
        _queue_name_prefix: Option<String>,
        _next_token: Option<String>,
    ) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query!(r#"SELECT name FROM queues LIMIT $1"#, max_results)
            .fetch_all(self.db_pool)
            .await?;

        let mut queue_urls: Vec<String> = Vec::new();
        rows.iter().for_each(|row| {
            queue_urls.push(format!("{}/{}", self.hostname, &row.name));
        });

        Ok(queue_urls)
    }

    pub fn send_message(&self) {
        todo!()
    }
}
