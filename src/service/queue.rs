use sqlx::SqlitePool;
use std::collections::HashMap;

pub struct Queue<'a> {
    db_pool: &'a SqlitePool,
    hostname: &'a str,
}

#[derive(Debug, Clone)]
pub struct QueueEntity {
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub name: String,
    pub queue_type: String,
    pub attributes: Option<HashMap<String, String>>,
    pub tags: Option<HashMap<String, String>>,
    #[allow(dead_code)]
    pub created_at: Option<time::OffsetDateTime>,
    #[allow(dead_code)]
    pub updated_at: Option<time::OffsetDateTime>,
}

impl QueueEntity {
    fn get_type(&self) -> String {
        if self.queue_type.contains(".fifo") {
            "Fifo".to_string()
        } else {
            "Standard".to_string()
        }
    }
}

impl<'a> Queue<'a> {
    pub fn new(db_pool: &'a SqlitePool, hostname: &'a str) -> Self {
        Queue { db_pool, hostname }
    }

    /// Create queue attributes in the database
    /// If the attribute exists, update the value
    /// Attributes come from the https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_CreateQueue.html
    async fn create_attributes(
        &self,
        queue_id: i64,
        attributes: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        for (key, value) in attributes {
            sqlx::query!(
                r#"
                INSERT INTO `attributes` (`queue_id`, `name`, `value`) 
                VALUES (?, ?, ?) 
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

    /// Create queue tags in the database
    /// If the tag exists, update the value
    /// Tags come from the https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_CreateQueue.html
    async fn create_tags(
        &self,
        queue_id: i64,
        tags: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        for (key, value) in tags {
            sqlx::query!(
                r#"
                INSERT INTO `tags` (`queue_id`, `name`, `value`) 
                VALUES (?, ?, ?) 
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
        let queue_type = queue.get_type().clone();
        let inserted_id = sqlx::query!(
            r#"
            INSERT INTO `queues` (`name`, `type`) 
            VALUES (?, ?) 
            "#,
            queue.name,
            queue_type
        )
        .execute(self.db_pool)
        .await?
        .last_insert_rowid();

        if let Some(attributes) = queue.attributes {
            self.create_attributes(inserted_id, attributes).await?;
        }

        if let Some(tags) = queue.tags {
            self.create_tags(inserted_id, tags).await?;
        }

        Ok(inserted_id.to_string())
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

    /// Check if a queue exists in the database by name.
    pub async fn queue_exists(&self, queue_name: &str) -> anyhow::Result<bool> {
        let row: Option<(i64,)> = sqlx::query_as(r#"SELECT id FROM queues WHERE name = ?"#)
            .bind(queue_name)
            .fetch_optional(self.db_pool)
            .await?;
        Ok(row.is_some())
    }

    /// Get all attributes for a queue from the database.
    pub async fn get_queue_attributes(
        &self,
        queue_name: &str,
    ) -> anyhow::Result<HashMap<String, String>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT a.name, a.value
            FROM attributes a
            JOIN queues q ON q.id = a.queue_id
            WHERE q.name = ?
            "#,
        )
        .bind(queue_name)
        .fetch_all(self.db_pool)
        .await?;

        let mut map = HashMap::new();
        for (name, value) in rows {
            map.insert(name, value);
        }
        Ok(map)
    }

    /// Set (upsert) attributes for a queue in the database.
    pub async fn set_queue_attributes(
        &self,
        queue_name: &str,
        attrs: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        // Look up queue id
        let (queue_id,): (i64,) = sqlx::query_as(r#"SELECT id FROM queues WHERE name = ?"#)
            .bind(queue_name)
            .fetch_one(self.db_pool)
            .await?;

        for (key, value) in attrs {
            // Try update first, then insert if no rows affected
            let result = sqlx::query(
                r#"
                UPDATE attributes SET value = ?
                WHERE queue_id = ? AND name = ?
                "#,
            )
            .bind(&value)
            .bind(queue_id)
            .bind(&key)
            .execute(self.db_pool)
            .await?;

            if result.rows_affected() == 0 {
                sqlx::query(
                    r#"
                    INSERT INTO attributes (queue_id, name, value)
                    VALUES (?, ?, ?)
                    "#,
                )
                .bind(queue_id)
                .bind(&key)
                .bind(&value)
                .execute(self.db_pool)
                .await?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn send_message(&self) {
        todo!()
    }
}
