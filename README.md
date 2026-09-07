# zhihu-cli

CLI for the [Zhihu Open Platform API](https://developer.zhihu.com).

Supports authentication, Zhihu search, global web search, the Zhida chat completion API, the hot list, quota queries, knowledge bases (list/items/upload/RAG search), PDF parsing, PPT generation, and Zhihu user data (contents/followees/collections/favlists).

## Installation

### From npm (recommended for Agent environments)

```bash
npm install -g @dawnswwwww/zhihu-cli
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

# Check remaining daily quota
zhihu quota --ids knowledge,tools

# Upload a file to your default knowledge base and RAG-search it
zhihu kb upload ./product-doc.pdf
zhihu kb search "退款规则是什么？" --scope personal

# One-shot PDF parse (upload → create task → wait for result)
zhihu pdf parse ./report.pdf

# Generate a PPT from a Zhihu answer and wait for the download link
zhihu ppt generate "https://www.zhihu.com/question/.../answer/..." --pages 12
```

## Claude Skill

This repository includes a Claude skill to help you use `zhihu-cli` from Claude Code or Claude.ai.

Install it with the [Skills CLI](https://skills.sh/):

```bash
# Install from GitHub
npx skills add dawnswwwww/zhihu-cli

# Or install globally
npx skills add dawnswwwww/zhihu-cli -g
```

Once installed, Claude will automatically use the skill whenever you ask about searching Zhihu, using the Zhida API, or configuring `zhihu-cli`.

## Commands

| Command | Description |
|---------|-------------|
| `zhihu auth login` | Interactive login (reads secret from stdin). |
| `zhihu auth set-secret <SECRET>` | Save Access Secret directly. |
| `zhihu auth status` | Show authentication status. |
| `zhihu search zhihu <QUERY>` | Search within Zhihu. |
| `zhihu search global <QUERY>` | Search the whole web. Use `--filter` and `--db`. |
| `zhihu ask <QUERY>` | Ask Zhida. Use `--model fast/thinking/agent` and `--stream`. |
| `zhihu hot` | Show the Zhihu hot list. `--limit` up to 30. |
| `zhihu quota` | Query daily free quota usage. `--ids` to filter API IDs. |
| `zhihu kb list` | List knowledge bases. `--scope all/created/subscribed`. |
| `zhihu kb items <KB_ID>` | List knowledge base contents. `--cursor`/`--limit` (max 20). |
| `zhihu kb upload <FILE>` | Upload a file to a knowledge base. `--kb-id` to target one. |
| `zhihu kb search <QUERY>` | RAG search. `--kb-id`/`--scope` (at least one), `--limit` (max 10). |
| `zhihu pdf upload <FILE>` | Upload a PDF for parsing; returns `file_id` (valid 24h). |
| `zhihu pdf task <FILE_ID>` | Create a PDF parse task. `--idempotency-key`. |
| `zhihu pdf status <TASK_ID>` | Query a PDF parse task. |
| `zhihu pdf parse <FILE>` | One-shot: upload + create task + wait for the result link. `--timeout-secs`. |
| `zhihu ppt task <URL> --pages N` | Create a PPT generation task from an answer/article URL. |
| `zhihu ppt status <TASK_ID>` | Query a PPT generation task. |
| `zhihu ppt generate <URL> --pages N` | One-shot: create task + wait for the PPTX link. `--timeout-secs`. |
| `zhihu user contents` | Your created contents. `--type`, `--limit` (max 50), `--sort-field`, `--sort-order`. |
| `zhihu user followees` | Users you follow. `--offset`/`--limit`. |
| `zhihu user collections` | Your recently collected contents. |
| `zhihu user favlists` | Your favlists. |
| `zhihu user favlist-contents <TOKEN>` | Contents of a favlist (token from `user favlists`). |
| `zhihu user <cmd> --oauth-token <T>` | Query an OAuth-authorized user's data instead of your own. |

Run `zhihu --help` or `zhihu <command> --help` for details.

## Configuration

The CLI stores configuration (including the Access Secret) at `~/.zhihu-cli/config.toml` (created with `0o600` permissions on Unix).

You can also override the secret at runtime with the `ZHIHU_ACCESS_SECRET` environment variable (it takes precedence over the config file), and point the CLI at another API host with `ZHIHU_OPENAPI_BASE_URL`.

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
git tag -a v0.1.3 -m "Release 0.1.3"
git push origin v0.1.3
```

This builds cross-platform binaries, creates a GitHub Release, publishes `zhihu-cli` to npm, and updates the Homebrew tap.

## License

[MIT](LICENSE)
