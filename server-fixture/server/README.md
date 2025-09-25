# tlsn-server-fixture

TLSNotary demo server fixture with dashboard UI for DevConnect booth display.

Inspired by `httpbin.org`.

# Quickstart

```bash
cargo run --release
```

## Dashboard UI

The server now serves a dashboard UI at `/` that shows:

- **Swiss Bank Demo** header with explanation
- **Bank Reserves** showing fake ETH, BTC, and USDC balances  
- **Live Access Log** displaying authorized/unauthorized requests to `/balances`
- Large fonts and high contrast colors suitable for booth display

Visit http://localhost:3000/ to view the dashboard.

## Setting the port

Set the enviroment variable `PORT` to configured the port the server runs on.

```bash
PORT=3001 cargo run --release
```

## Logging

Enable server logs by setting the log level:

```bash
RUST_LOG=info cargo run --release
```

## Testing

You can test the server works using curl:

```bash
curl https://0.0.0.0:3000/formats/html --insecure
```

Notice the `--insecure` flag, which will ignore that the server presents a self-signed cert.

# Formats

## JSON

The `/json` endpoint provides JSON data fixtures. You can pass the `size` query parameter to select between the 3 available payload sizes which are 1Kb, 4Kb, 8Kb.

```bash
curl https://0.0.0.0:3000/formats/json?size=4 --insecure
```

## Bank Balances Endpoint

The server provides a `/balances` endpoint that returns fake bank balance data:

```bash
# Unauthorized access (will be logged as "unauthorized")
curl https://0.0.0.0:3000/balances --insecure

# Authorized access (will be logged as "authorized") 
curl https://0.0.0.0:3000/balances --insecure -H "Authorization: Bearer random_auth_token"
```

All access attempts are logged and displayed in real-time on the dashboard UI.

## Access Log API

The server exposes the access log at `/log` as JSON:

```bash
curl https://0.0.0.0:3000/log --insecure
```

## Browser Kiosk Mode

For Raspberry Pi deployment, run Chromium in kiosk mode:

```bash
chromium-browser --kiosk http://localhost:3000/
```