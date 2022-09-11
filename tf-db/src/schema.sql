CREATE TABLE IF NOT EXISTS tracks (
	id TEXT PRIMARY KEY,
	artist TEXT NOT NULL,
	title TEXT NOT NULL,
	source TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
	id TEXT PRIMARY KEY,
	name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS track_tags (
	track_id TEXT NOT NULL,
	tag_id TEXT NOT NULL,
	"value" FLOAT NOT NULL,
	PRIMARY KEY (track_id, tag_id)
);

CREATE VIRTUAL TABLE IF NOT EXISTS tag_search USING fts5(name, tokenize="trigram");

INSERT INTO tag_search
	SELECT name FROM tags src
	WHERE NOT EXISTS (
		SELECT name
		FROM tag_search dst
		WHERE src.name = dst.name
	);

