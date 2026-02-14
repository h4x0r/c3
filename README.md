# C3 - Claude Code Chat over Signal

Chat with Claude AI directly from Signal. Send a message, get an intelligent response — right in your favourite messaging app.

## What is C3?

C3 turns your Signal messenger into a Claude AI chat interface. Instead of opening a browser or a separate app, just text your questions and Claude replies in Signal.

- Ask questions, get answers — in Signal
- Each person gets their own private conversation that remembers context
- Choose between different Claude models (Opus, Sonnet, Haiku)
- Built-in cost tracking so you stay within budget

## How it Works

```
You (Signal) → C3 → Claude AI → C3 → You (Signal)
```

You send a message on Signal. C3 picks it up, asks Claude, and sends the answer back. That's it.

## Quick Start

### What You Need

1. **A Signal account** linked to [signal-cli](https://github.com/AsamK/signal-cli)
2. **[signal-cli-api](https://github.com/h4x0r/signal-cli-api)** running on your machine
3. **[Claude Code](https://docs.anthropic.com/en/docs/claude-code)** installed and logged in
4. **Rust** installed ([rustup.rs](https://rustup.rs))

### Install and Run

```bash
# Download and build
git clone https://github.com/h4x0r/c3.git
cd c3
cargo build --release

# Run (replace with your actual numbers)
./target/release/c3 --account +447700000000 --allowed +447700000001
```

- `--account` is the Signal number C3 listens on
- `--allowed` controls who can use it (comma-separated numbers, or leave it out to allow everyone)

### Using Environment Variables

Instead of typing flags every time, you can set environment variables:

```bash
export C3_ACCOUNT=+447700000000
export C3_ALLOWED=+447700000001
./target/release/c3
```

Copy `.env.example` to `.env` for a template.

## Using C3

Once running, just send a message to your C3 Signal number from your phone. Claude will respond in the same chat.

### Special Commands

Type these in Signal to control C3:

| Command | What it Does |
|---------|-------------|
| `/reset` | Start a fresh conversation (clears memory) |
| `/status` | Show how long C3 has been running, message count, and total cost |
| `/model sonnet` | Switch to a different Claude model (opus, sonnet, haiku) |

Everything else you type gets sent to Claude.

## Options

| Setting | Default | What it Does |
|---------|---------|-------------|
| `--account` | (required) | Your Signal number |
| `--allowed` | everyone | Who's allowed to chat |
| `--model` | opus | Which Claude model to use |
| `--max-budget` | $5.00 | Maximum spend per message |
| `--api-url` | localhost:8080 | Where signal-cli-api is running |

## How Much Does it Cost?

C3 itself is free. You pay for Claude API usage through your Anthropic subscription. Use `/status` in Signal to check your running total, and `--max-budget` to set a per-message spending cap.

## Troubleshooting

**C3 starts but I don't get replies**
- Make sure signal-cli-api is running (`curl http://localhost:8080/v1/health`)
- Check that `claude` works on its own (`claude -p "hello"`)
- Verify your Signal number is correct

**"Ignoring message from non-allowed sender"**
- The sender's number isn't in your `--allowed` list. Add it or remove the `--allowed` flag entirely.

**Messages are cut off**
- Long responses are automatically split into multiple Signal messages. They should arrive in order.

## License

MIT
