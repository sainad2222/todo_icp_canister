# `todo_app`

Todo app backend canister

## Prerequesities
- rust
- candid-extractor(`cargo install candid-extractor`)
- dfx

## Build instructions
- dfx build
- candid-extractor target/wasm32-unknown-unknown/release/todo_app_backend.wasm > src/todo_app_backend/todo_app_backend.did

## How to run?
- dfx start(not needed for mainnet)
- dfx deploy(use ic flag for mainnet)
