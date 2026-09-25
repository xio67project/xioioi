CREATE TABLE problems (
	id INTEGER PRIMARY KEY,
	name TEXT NOT NULL,
	hash TEXT NOT NULL,
	title TEXT NOT NULL,
	time_limit INTEGER NOT NULL,
	memory_limit INTEGER NOT NULL,
	public INTEGER NOT NULL DEFAULT 0,
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	UNIQUE (name, hash)
);

CREATE VIEW problem_view AS
SELECT id AS key, name || '#' || hash AS id, name, title, time_limit, memory_limit, public, created_at FROM problems;
