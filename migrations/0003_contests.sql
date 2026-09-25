CREATE TABLE contests (
	id INTEGER PRIMARY KEY,
	slug TEXT NOT NULL UNIQUE,
	name TEXT NOT NULL,
	start_at TEXT NOT NULL,
	end_at TEXT,
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE contest_problems (
	contest_id INTEGER NOT NULL REFERENCES contests(id) ON DELETE CASCADE,
	problem_id INTEGER NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
	label TEXT NOT NULL,
	PRIMARY KEY (contest_id, problem_id),
	UNIQUE (contest_id, label)
);

CREATE VIEW contest_problem_view AS
SELECT cp.contest_id, cp.label, p.id, p.title
FROM contest_problems cp
JOIN problem_view p ON p.key = cp.problem_id;
