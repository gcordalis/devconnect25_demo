Build:
```
docker build -t swissbank-server .
```
Run:
```
docker run -p 3000:3000 swissbank-server
```


Test:
```
curl http://localhost:3000/balances --insecure -H "Authorization: Bearer random_auth_token"
```
Should output:
{"accounts":{"CHF":"50_000_000","EUR":"275_000_000","USD":"125_000_000"},"auditor":"PwC","bank":"Swiss Bank","last_audit":"2025-11-12T12:00:00Z","organization":"Ethereum Foundation"}%   