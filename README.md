# xioioi

## Requirements

- Rust
- `tailwindcss`, `esbuild`, `tsc` in `PATH` (`npm i -g @tailwindcss/cli esbuild typescript`)

## Running

```sh
cargo run
```

http://localhost:8080

## Layout

| Path | What |
|---|---|
| `src/main.rs` | entry point |
| `src/router.rs` | serves backend + frontend |
| `src/backend/backend.rs` | `/api/*` |
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
