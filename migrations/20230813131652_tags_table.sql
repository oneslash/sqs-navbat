CREATE TABLE IF NOT EXISTS tags (
	id INTEGER,
	name TEXT NOT NULL,
	value TEXT NOT NULL,
	queue_id INTEGER NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(id AUTOINCREMENT)
);
