## Why

Tool handling (`FindTool`, `ChangeTool`, `SetTool`, `AlignTool`, `EnumTools` and friends) was added as validated typed commands, with mock replies and TUI presets.

## What Changes

- Tool commands and validated `ToolName`, `UnitVector`, `ToolAlignment` (`typed-commands`).
- Mock tool replies (`mock-server`).
- TUI presets for tool commands (`tui`).

## Capabilities

### Modified Capabilities

- `typed-commands`
- `mock-server`
- `tui`

## Impact

Retrospective change: it records behavior introduced by git commit(s) `f214d69`. No code changes.
