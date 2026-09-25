#!/usr/bin/env python3
import base64
import getpass
import hashlib
import io
import os
import re
import sqlite3
import sys
import tomllib
import zipfile

ROOT = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(ROOT, "data")
DB = os.path.join(DATA, "xioioi.db")
PROBLEMS = os.path.join(DATA, "problems")

USAGE = """usage:
  ./db.py add user <name>
  ./db.py add admin <name>
  ./db.py add problem <file.zip|folder> [public]
  ./db.py add contest <slug> <name> <start> [end]
  ./db.py link <contest> <label> <problem-id>
  ./db.py unlink <contest> <label>
  ./db.py public <problem-id>
  ./db.py private <problem-id>
  ./db.py del user <name>
  ./db.py del problem <problem-id>
  ./db.py del contest <slug>
  ./db.py list users|problems|contests
  ./db.py list contest <slug>

problem: <name>.zip or folder <name>/
  config.toml   title = "Sum", time_limit = 1000 (ms), memory_limit = 262144 (KiB)
  DOC.md        statement
  in/a00.in     test a00 (letter = group, _ = example shown in the statement)
  out/a00.out

dates: "YYYY-MM-DD HH:MM:SS\""""

LN, R, P = 15, 8, 1


def b64(data):
	return base64.b64encode(data).decode().rstrip("=")


def hash_password(password):
	salt = os.urandom(16)
	key = hashlib.scrypt(password.encode(), salt=salt, n=1 << LN, r=R, p=P, maxmem=64 * 1024 * 1024, dklen=32)
	return f"$scrypt$ln={LN},r={R},p={P}${b64(salt)}${b64(key)}"


def ask_password():
	first = getpass.getpass("password: ")
	second = getpass.getpass("again: ")
	if first != second:
		sys.exit("passwords don't match")
	if not first:
		sys.exit("password can't be empty")
	return first


def connect():
	if not os.path.exists(DB):
		sys.exit(f"{DB} doesn't exist, start xioioi once to create it")
	con = sqlite3.connect(DB)
	con.execute("PRAGMA foreign_keys = ON")
	found = con.execute("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'contest_problems'").fetchone()
	if not found:
		sys.exit("database is out of date, start xioioi once to update it")
	return con


def problem_key(con, id):
	row = con.execute("SELECT key FROM problem_view WHERE id = ?", (id,)).fetchone()
	if not row:
		sys.exit(f"no problem {id}")
	return row[0]


def contest_key(con, slug):
	row = con.execute("SELECT id FROM contests WHERE slug = ?", (slug,)).fetchone()
	if not row:
		sys.exit(f"no contest {slug}")
	return row[0]


def add_user(con, kind, name):
	password = hash_password(ask_password())
	try:
		with con:
			con.execute("INSERT INTO users (name, password, admin) VALUES (?, ?, ?)", (name, password, kind == "admin"))
	except sqlite3.IntegrityError:
		sys.exit(f"user {name} already exists")
	print(f"added {kind} {name}")


TEST = re.compile(r"^([_a-z])(\d+)$")


def check_package(z):
	files = [n for n in z.namelist() if not n.endswith("/")]
	tops = {n.split("/")[0] for n in files}
	prefix = ""
	if len(tops) == 1 and "config.toml" not in files:
		prefix = tops.pop() + "/"
	files = {n[len(prefix):] for n in files if n.startswith(prefix)}

	for need in ("config.toml", "DOC.md"):
		if need not in files:
			sys.exit(f"{need} missing in the zip")
	cfg = tomllib.loads(z.read(prefix + "config.toml").decode())
	for key in ("title", "time_limit", "memory_limit"):
		if key not in cfg:
			sys.exit(f"config.toml is missing {key}")

	ins = {n[3:-3] for n in files if n.startswith("in/") and n.endswith(".in")}
	outs = {n[4:-4] for n in files if n.startswith("out/") and n.endswith(".out")}
	for test in sorted(ins ^ outs):
		kind = "out" if test in ins else "in"
		sys.exit(f"test {test} has no {kind}/{test}.{kind}")
	for test in sorted(ins):
		if not TEST.match(test):
			sys.exit(f"bad test name {test}, use a letter (group) or _ (example) + digits, e.g. a00, _00")
	if not any(t[0] != "_" for t in ins):
		sys.exit("no tests, only examples")
	return cfg, prefix


def pack(folder):
	buf = io.BytesIO()
	with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
		for root, dirs, files in os.walk(folder):
			dirs[:] = sorted(d for d in dirs if not d.startswith("."))
			for f in sorted(files):
				if f.startswith("."):
					continue
				full = os.path.join(root, f)
				info = zipfile.ZipInfo(os.path.relpath(full, folder).replace(os.sep, "/"), date_time=(1980, 1, 1, 0, 0, 0))
				info.external_attr = 0o644 << 16
				info.compress_type = zipfile.ZIP_DEFLATED
				with open(full, "rb") as src:
					z.writestr(info, src.read(), compresslevel=9)
	return buf.getvalue()


def add_problem(con, path, public):
	path = path.rstrip("/")
	if os.path.isdir(path):
		data = pack(path)
	elif zipfile.is_zipfile(path):
		with open(path, "rb") as f:
			data = f.read()
	else:
		sys.exit(f"{path} isn't a zip file or a folder")

	name = os.path.splitext(os.path.basename(path))[0]
	if not re.fullmatch(r"[A-Za-z0-9_-]+", name):
		sys.exit(f"bad problem name {name}, use letters, digits, _ and -")
	hash = hashlib.sha1(data).hexdigest()[:7]
	id = f"{name}#{hash}"

	with zipfile.ZipFile(io.BytesIO(data)) as z:
		cfg, _ = check_package(z)

	dest = os.path.join(PROBLEMS, id + ".zip")
	if os.path.exists(dest):
		sys.exit(f"problem {id} already exists")
	try:
		with con:
			con.execute(
				"INSERT INTO problems (name, hash, title, time_limit, memory_limit, public) VALUES (?, ?, ?, ?, ?, ?)",
				(name, hash, cfg["title"], cfg["time_limit"], cfg["memory_limit"], public),
			)
			os.makedirs(PROBLEMS, exist_ok=True)
			with open(dest, "wb") as f:
				f.write(data)
	except sqlite3.IntegrityError:
		sys.exit(f"problem {id} already exists")
	print(f"added problem {id} ({'public' if public else 'private'})")


def add_contest(con, slug, name, start, end):
	try:
		with con:
			con.execute("INSERT INTO contests (slug, name, start_at, end_at) VALUES (?, ?, ?, ?)", (slug, name, start, end))
	except sqlite3.IntegrityError:
		sys.exit(f"contest {slug} already exists")
	print(f"added contest {slug}")


def link(con, slug, label, id):
	try:
		with con:
			con.execute(
				"INSERT INTO contest_problems (contest_id, problem_id, label) VALUES (?, ?, ?)",
				(contest_key(con, slug), problem_key(con, id), label),
			)
	except sqlite3.IntegrityError:
		sys.exit(f"{slug} already has label {label} or problem {id}")
	print(f"{slug} {label} = {id}")


def unlink(con, slug, label):
	with con:
		cur = con.execute("DELETE FROM contest_problems WHERE contest_id = ? AND label = ?", (contest_key(con, slug), label))
	if cur.rowcount == 0:
		sys.exit(f"{slug} has no label {label}")
	print(f"removed {label} from {slug}")


def set_public(con, id, public):
	with con:
		con.execute("UPDATE problems SET public = ? WHERE id = ?", (public, problem_key(con, id)))
	print(f"{id} is now {'public' if public else 'private'}")


def delete(con, kind, what):
	if kind == "user":
		with con:
			cur = con.execute("DELETE FROM users WHERE name = ?", (what,))
		if cur.rowcount == 0:
			sys.exit(f"no user {what}")
	elif kind == "problem":
		key = problem_key(con, what)
		with con:
			con.execute("DELETE FROM problems WHERE id = ?", (key,))
		zip = os.path.join(PROBLEMS, what + ".zip")
		if os.path.exists(zip):
			os.remove(zip)
	elif kind == "contest":
		key = contest_key(con, what)
		with con:
			con.execute("DELETE FROM contests WHERE id = ?", (key,))
	print(f"deleted {kind} {what}")


def show(con, what):
	if what == "users":
		rows = con.execute("SELECT id, name, CASE admin WHEN 1 THEN 'admin' ELSE 'user' END, created_at FROM users ORDER BY id")
	elif what == "problems":
		rows = con.execute("SELECT id, title, CASE public WHEN 1 THEN 'public' ELSE 'private' END, time_limit, memory_limit FROM problem_view ORDER BY name")
	elif what == "contests":
		rows = con.execute("SELECT slug, name, start_at, coalesce(end_at, '-') FROM contests ORDER BY start_at DESC")
	for row in rows:
		print("\t".join(str(x) for x in row))


def show_contest(con, slug):
	for row in con.execute("SELECT label, id, title FROM contest_problem_view WHERE contest_id = ? ORDER BY label", (contest_key(con, slug),)):
		print("\t".join(row))


def main():
	a = sys.argv[1:]
	match a:
		case ["add", ("user" | "admin") as kind, name]:
			add_user(connect(), kind, name)
		case ["add", "problem", path]:
			add_problem(connect(), path, False)
		case ["add", "problem", path, "public"]:
			add_problem(connect(), path, True)
		case ["add", "contest", slug, name, start]:
			add_contest(connect(), slug, name, start, None)
		case ["add", "contest", slug, name, start, end]:
			add_contest(connect(), slug, name, start, end)
		case ["link", slug, label, id]:
			link(connect(), slug, label, id)
		case ["unlink", slug, label]:
			unlink(connect(), slug, label)
		case ["public", id]:
			set_public(connect(), id, True)
		case ["private", id]:
			set_public(connect(), id, False)
		case ["del", ("user" | "problem" | "contest") as kind, what]:
			delete(connect(), kind, what)
		case ["list", ("users" | "problems" | "contests") as what]:
			show(connect(), what)
		case ["list", "contest", slug]:
			show_contest(connect(), slug)
		case _:
			sys.exit(USAGE)


if __name__ == "__main__":
	main()
