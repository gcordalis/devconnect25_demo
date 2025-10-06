# Prover Server

WebSocket server that acts as the TLSNotary prover in the DevConnect demo.

## Running the server
1. Configure this server setting via the global variables defined in [main.rs](./src/main.rs) — please ensure that the hardcoded `SERVER_DOMAIN` has the same value on the prover side.
2. Start the server by running the following in a terminal at the root of this crate.
```bash
cargo run --release
```

## WebSocket APIs
### /prove
For prover connections via websocket, i.e. `ws://localhost:9816/prove`

### /verify
For verification via websocket, i.e. `ws://localhost:9816/verify`
