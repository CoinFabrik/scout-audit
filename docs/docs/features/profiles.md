---
sidebar_position: 20
---

# Profile configuration

The profile configuration file is generated automatically in `$HOME/.config/scout/soroban-config.toml` the first time scout-audit is run.
The configuration has the following format

```toml
[<profile-name>.<detector-name>]
enabled = <true|false>
```

For example, if you want to define a profile named 'dev' in which the 'panic-error' detector is disabled and the 'soroban-version' detector is enabled, you should do the following:

```toml
[dev.panic-error]
enabled = false
[dev.soroban-version]
enabled = true
```
