# Testing Strategy

## Unit Tests

Located alongside code (`#[cfg(test)] mod tests`):

- `event.rs`: `extract_text_delta`, `extract_completion` with various event formats
- `session/message.rs`: `parse_message_list` for both wrapped and array formats

## Integration Tests

Located in `tests/`:

- `detect_test.rs`: Platform-specific command detection (BDD-style)

## Test Style

Following BDD conventions with descriptive comments:

```rust
/// Given an absolute path that exists,
/// When detect_command is called with it,
/// Then available is true and path is Some.
#[test]
fn absolute_path_exists_returns_available() {
    // ...
}
```
