-- Add migration script here
CREATE TABLE "new_queues" (
	"id"	INTEGER,
	"name"	TEXT NOT NULL,
	"tag_name"	TEXT NOT NULL,
	"tag_value"	TEXT NOT NULL,
	"created_at"	TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"updated_at"	TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"type"	TEXT NOT NULL DEFAULT '',
	PRIMARY KEY("id" AUTOINCREMENT)
);

INSERT INTO new_queues SELECT * FROM queues;

DROP TABLE queues;

ALTER TABLE new_queues RENAME TO queues;
