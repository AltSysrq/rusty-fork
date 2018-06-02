## 0.2.0

### Breaking changes

- APIs which used to provide a `std::process::Child` now instead provide a
  `rusty_fork::ChildWrapper`.

### Bug fixes

- Fix that using the "timeout" feature, or otherwise using `wait_timeout` on
  the child process, could cause an unrelated process to get killed if the
  child exits within the timeout.

## 0.1.1

### Minor changes

- `tempfile` updated to 3.0.
