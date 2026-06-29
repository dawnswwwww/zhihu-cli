# zhihu-cli

CLI for the [Zhihu Open Platform API](https://developer.zhihu.com).

Supports authentication, Zhihu search, global web search, and the Zhida chat completion API.

## Installation

### From npm (recommended for Agent environments)

```bash
npm install -g zhihu-cli
zhihu --help
```

### From Homebrew

```bash
brew tap dawnswwwww/tap
brew install zhihu-cli
```

### From crates.io

```bash
cargo install zhihu-cli
```

### From GitHub Releases

Download the prebuilt archive for your platform from the [Releases](https://github.com/dawnswwwww/zhihu-cli/releases) page, extract it, and place the `zhihu` binary in your `PATH`.

## Quick start

```bash
# Authenticate with your Access Secret
zhihu auth login

# Or set it directly
zhihu auth set-secret <YOUR_ACCESS_SECRET>

# Search Zhihu
zhihu search zhihu "Rust 入门" --count 5

# Search the whole web
zhihu search global "Rust 入门" --count 5

# Ask Zhida
zhihu ask "Rust 和 Go 怎么选？" --model thinking
```

## Commands

| Command | Description |
|---------|-------------|
| `zhihu auth login` | Interactive login (reads secret from stdin). |
| `zhihu auth set-secret <SECRET>` | Save Access Secret directly. |
| `zhihu auth status` | Show authentication status. |
| `zhihu search zhihu <QUERY>` | Search within Zhihu. |
| `zhihu search global <QUERY>` | Search the whole web. |
| `zhihu ask <QUERY>` | Ask Zhida. Use `--model fast/thinking/agent` and `--stream`. |

Run `zhihu --help` or `zhihu <command> --help` for details.

## Configuration

The CLI stores configuration (including the Access Secret) under the user's config directory, typically:

- macOS/Linux: `~/.config/zhihu-cli/config.toml`
- Windows: `%APPDATA%\zhihu-cli\config.toml`

You can also override the secret at runtime with the `ZHIHU_ACCESS_SECRET` environment variable.

## Development

```bash
# Run tests
make test

# Lint
make lint

# Full local CI check
make check
```

See [docs/development.md](docs/development.md) for the full workflow.

## Release

Pushing a SemVer tag triggers the release workflow:

```bash
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin v0.1.0
```

This builds cross-platform binaries, creates a GitHub Release, publishes `zhihu-cli` to npm, and updates the Homebrew tap.

## License

[MIT](LICENSE)
