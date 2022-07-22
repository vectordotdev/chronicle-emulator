# Chronicle Emulator

This is a simple web server that emulates the API of Google Chronicle. It is
used for integration testing in Vector.

The server is hardcoded to listen on port 3000.

## Features

### Authentication

Authentication is via a JWT token. The public key is passed by specifying a
file as an arg. Eg.

  ```sh
  >  cat public.txt
-----BEGIN PUBLIC KEY-----
...
-----END PUBLIC KEY-----

> ./target/release/chronicle-emulator -p ./public.txt
```


### Log types

The `/v2/logtypes` endpoint is supported and returns a hardcoded limited list
of supported logtypes.

*There is no validation to ensure that the log type passed when posting log entries
exists in this list.* This is because it is useful for a test to generate a
completely random log type which can be used to query the successfully posted
log entries later.

If you want to invoke an invalid log type error when posting a log entry, use
a logtype of `"INVALID"`.

### Unstructured log entries

The `/v2/unstructuredlogentries:batchCreate` endpoint is supported. The posted json
must have this structure:

```json
{
  "customer_id": "...",
  "log_type": "...",
  "entries": [ { "log_text": "...." } ]
}
```

### Queries

For integration tests to fetch the log entries that have been successfully posted
an endpoint `/logs?log_type=...` is provided. It will return all log entries with
the given log type that has been posted. Note, the entries are just held in memory
so the state is reset when the program restarts.

To query all posted logs, just omit the `log_type` parameter.
