## 1. Implementation

- [x] 1.1 `leading_tag` reads a tag from a raw line (`wire-format`).
- [x] 1.2 The encoder can write a raw line verbatim and still refuses CR/LF (`line-codec`).
- [x] 1.3 `IppClient::send_line` (`ipp-client`).

## 2. Verification

- [x] 2.1 Behavior matches the code and tests at `f52187c`
