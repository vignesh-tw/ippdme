## 1. Implementation

- [x] 1.1 `Handler::Session` created per connection; handlers answer with an action (reply, raw line, close) (`server-transport`).
- [x] 1.2 `drop: true` and `malformed: "text"` response kinds (`imposter`).

## 2. Verification

- [x] 2.1 Behavior matches the code and tests at `a5620ad`
