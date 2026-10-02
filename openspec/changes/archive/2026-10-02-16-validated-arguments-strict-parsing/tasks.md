## 1. Implementation

- [x] 1.1 `Command::try_from` fails on unknown commands; `from_term_lenient` opts in to `Raw` (`typed-commands`).
- [x] 1.2 Validated points, names, whole-number arguments and transformations (`typed-commands`).
- [x] 1.3 Invalid arguments surface as `ValueError` in Python (`python-bindings`).
- [x] 1.4 Client helpers fail before sending on invalid arguments (`ipp-client`).

## 2. Verification

- [x] 2.1 Behavior matches the code and tests at `25f89ed`
