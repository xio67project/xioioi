# xioioi

## Requirements

- Rust
- `tailwindcss`, `esbuild`, `tsc` in `PATH` (standalone [tailwindcss](https://github.com/tailwindlabs/tailwindcss/releases), `npm i -g esbuild typescript`)

## Running

```sh
cargo run
```

http://localhost:8080

## Database

```sh
./db.py add admin <name>
./db.py add problem examples/dol public
./db.py add contest r1 "Round 1" "2026-10-01 18:00:00"
./db.py link r1 A sum#741cf79
```

Run `./db.py` for all commands. Start xioioi once first so the database exists.

A problem id is `<zip name>#<first 7 chars of its sha1sum>`, its page is `/p/<name>/<hash>`.

Problem zip:

```
config.toml    title, time_limit (ms), memory_limit (KiB)
DOC.md         statement (Markdown)
in/_00.in      example, shown under the statement
out/_00.out
in/a00.in      test 00 of group a
out/a00.out
```

`add problem` takes a zip or a folder (zipped automatically, same folder = same hash). Problems are stored as `data/problems/<name#hash>.zip`. Examples are in `examples/`.

## Layout

| Path | What |
|---|---|
| `src/main.rs` | entry point, serves backend + frontend |
| `db.py` | manage users, problems, contests |
| `migrations/` | database schema |
| `examples/` | example problem packages |
| `data/` | runtime: `xioioi.db` + `problems/<name#hash>.zip` |
| `src/backend/backend.rs` | `/api/*` |
| `src/backend/db.rs` | SQLite database |
| `src/backend/auth.rs` | login sessions |
| `src/backend/pkg.rs` | reads problem packages |
| `src/frontend/frontend.rs` | `/` and `/static/*` |
| `src/frontend/web/templates/` | HTML |
| `src/frontend/web/ts/` | TypeScript |
| `src/frontend/web/tailwind.css` | styles |
| `src/frontend/web/static/` | static files |
| `build.rs` | builds CSS + TS |

## Verified commits

`main` requires signed commits.

```sh
ssh-keygen -t ed25519
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
gh auth refresh -s admin:ssh_signing_key
gh ssh-key add ~/.ssh/id_ed25519.pub --type signing
```

Your `user.email` must be verified on GitHub.
