[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

# lazydcompose

A terminal UI for managing multiple Docker Compose projects at a glance.

Browse projects and their containers, start/stop/restart from the keyboard,
tail logs, drop into a shell, and inspect networks, volumes, mounts and ports
— all without leaving the terminal.

## Install

```bash
cargo install --path .
```

Requires the `docker` CLI on your `PATH`.

## Usage

```bash
lazydcompose
```

The UI shows every Compose project Docker knows about. Press `?` at any time
to see the keybindings available in the current context.

## Configuration

Optional. Drop a `config.toml` at:

- Linux: `~/.config/lazydcompose/config.toml`
- macOS: `~/Library/Application Support/lazydcompose/config.toml`
- Windows: `%LOCALAPPDATA%\lazydcompose\config.toml`

Any of the actions below can be rebound. Keys are strings like `"ctrl+f"` or
`"shift+enter"`. The full default config:

```toml
[keybindings]
Quit                   = ["q", "ctrl+c"]
ShowBindings           = ["?"]

MoveUp                 = ["k", "up"]
MoveDown               = ["j", "down"]
Select                 = ["enter"]
Deselect               = ["esc"]

Info                   = ["i"]
QuitInfo               = ["esc"]
ScrollUp               = ["pageup"]
ScrollDown             = ["pagedown"]

DockerComposeUp        = ["u"]
DockerComposeDown      = ["d"]
DockerComposeStart     = ["s"]
DockerComposeStop      = ["shift+s"]
DockerComposeRestart   = ["r"]

DockerContainerStart   = ["s"]
DockerContainerStop    = ["shift+s"]
DockerContainerRestart = ["r"]
DockerFollowLogs       = ["m"]
DockerConsole          = ["shift+e"]
```

## License

MIT — see [LICENSE](./LICENSE).
