## ADDED Requirements
### Requirement: Command-line tool
`ippdme-tap --target HOST:PORT [--listen ADDR:PORT]` SHALL listen on `127.0.0.1:1297` by default, print the listen address, and print each event as `[#conn] opened from PEER`, `[#conn] client -> server  LINE`, `[#conn] server -> client  LINE` and `[#conn] closed`. A missing `--target`, an unknown flag or a flag without a value SHALL print usage and exit with status 2. Failing to bind SHALL print the error and exit with status 1.

#### Scenario: Missing target
- **WHEN** `ippdme-tap` is run without `--target`
- **THEN** usage is printed and the exit status is 2

