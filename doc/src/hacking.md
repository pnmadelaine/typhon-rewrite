# Hacking

Typhon is written in Rust and consists of several crates (located in `/workspace`).
- `typhon-actors`
- `typhon-nix`
- `typhon-core`
- `typhon-app`
- `typhon-server`
- `typhon-pack`

## Development environment

This documentation assumes that you are using Nix, so you can simply run
`nix-shell` at the root of the project to enter the development environment.
Experimental feature "nix-command" needs to be enabled in your Nix
configuration for the server to run properly.

The following instructions assume that you are inside the Nix development
environment.

## Dependencies

Typhon's main dependencies are:
- [Axum](https://github.com/tokio-rs/axum/): backend web framework
- [Diesel](https://diesel.rs/): ORM (database management)
- [Leptos](https::/leptos.dev/): frontend web framework
- [Tokio](https://tokio.rs/): asynchronous runtime
- [nix-daemon](https://crates.io/crates/nix-daemon): wrapper around the Nix daemon

## Nix shell commands

The Nix shell provides a few commands:

- `build` to compile
- `pg-start` and `pg-stop` to spawn the database
- `serve` to run the server
- `fmt` to format the code

By default the server is listening at `http://localhost:3000`.
