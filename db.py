#!/usr/bin/env python3
import base64
import getpass
import hashlib
import os
import sqlite3
import sys

DB = os.path.join(os.path.dirname(os.path.abspath(__file__)), "data", "xioioi.db")

USAGE = """usage:
  ./db.py add user <name>
  ./db.py add admin <name>
  ./db.py del <name>
  ./db.py list"""

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
	found = con.execute("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'users'").fetchone()
	if not found:
		sys.exit("no users table, start xioioi once to update the database")
	return con


def add(con, kind, name):
	password = hash_password(ask_password())
	try:
		with con:
			con.execute("INSERT INTO users (name, password, admin) VALUES (?, ?, ?)", (name, password, kind == "admin"))
	except sqlite3.IntegrityError:
		sys.exit(f"user {name} already exists")
	print(f"added {kind} {name}")


def delete(con, name):
	with con:
		cur = con.execute("DELETE FROM users WHERE name = ?", (name,))
	if cur.rowcount == 0:
		sys.exit(f"no user {name}")
	print(f"deleted {name}")


def show(con):
	for id, name, admin, created in con.execute("SELECT id, name, admin, created_at FROM users ORDER BY id"):
		role = "admin" if admin else "user"
		print(f"{id}\t{name}\t{role}\t{created}")


def main():
	args = sys.argv[1:]
	if len(args) == 3 and args[0] == "add" and args[1] in ("user", "admin"):
		add(connect(), args[1], args[2])
	elif len(args) == 2 and args[0] == "del":
		delete(connect(), args[1])
	elif args == ["list"]:
		show(connect())
	else:
		sys.exit(USAGE)


if __name__ == "__main__":
	main()
