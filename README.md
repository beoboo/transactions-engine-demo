# Transaction Engine

This project aims to be a Rust showcase.

Run 

```bash
cargo run transactions.csv > accounts.csv
```

to generate the accounts CSV from standard output. 

Errors will be reported to standard error (if any).

## Testing

The project contains unit tests, that can be run with the usual

```bash
cargo test
```

## Extended engine
In order to show how we could cache or externalize accounts data, 
the extended engine can be enabled with:

```bash
cargo run --features=extended transactions.csv > accounts.csv
```

This extended version contains a method (analyze_transaction) that 
could be used to stream transactions.

NB: extended engine is not unit tested, but tests can be easily ported.