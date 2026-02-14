# ccchat - Claude Code Chat

Chat with Claude AI directly from your favourite messenger. Send a message, get an intelligent response — no browser or app switching required.

## What is ccchat?

ccchat brings Claude AI into your messaging apps. Instead of opening a separate tool, just text your questions and Claude replies in the same chat.

- Ask questions, get answers — right where you message
- Each person gets their own private conversation that remembers context
- Choose between different Claude models (Opus, Sonnet, Haiku)
- Built-in cost tracking so you stay within budget

## Supported Messengers

| Messenger | Status |
|-----------|--------|
| Signal | Supported |

More messengers coming soon.

## How it Works

```
You (messenger) → ccchat → Claude AI → ccchat → You (messenger)
```

You send a message. ccchat picks it up, asks Claude, and sends the answer back. That's it.

## Quick Start (Signal)

### What You Need

1. **A Signal account** linked to [signal-cli](https://github.com/AsamK/signal-cli)
2. **[Claude Code](https://docs.anthropic.com/en/docs/claude-code)** installed and logged in

That's it. ccchat automatically installs and manages [signal-cli-api](https://github.com/h4x0r/signal-cli-api) for you.

### Install

```bash
cargo install ccchat
```

### Run

```bash
# Replace with your actual number
ccchat --account +447700000000
```

On first start, ccchat only accepts messages from your own number (Note to Self). When someone new messages you, ccchat sends you a notification via Note to Self with their name and a one-tap `/allow` command.

You can also use environment variables instead of flags:

```bash
export CCCHAT_ACCOUNT=+447700000000
ccchat
```

Copy `.env.example` to `.env` for a template.

## Using ccchat

Once running, just send a message to your ccchat number from your phone. Claude will respond in the same chat.

### Sender Approval

By default, only you (the account owner) can chat. When someone else sends a message:

1. ccchat blocks the message and notifies you via Note to Self
2. The notification includes their name and a ready-to-use `/allow` command
3. Reply `/allow <id>` to approve them — the approval is saved permanently

Approved senders are stored in `~/.config/ccchat/allowed.json` and persist across restarts.

### Commands

Type these in your chat to control ccchat:

| Command | What it Does |
|---------|-------------|
| `/allow <id>` | Permanently approve a sender |
| `/revoke <id>` | Remove a sender's access |
| `/pending` | Show blocked senders waiting for approval |
| `/status` | Show uptime, message count, active sessions, and total cost |
| `/model sonnet` | Switch to a different Claude model (opus, sonnet, haiku) |
| `/reset` | Start a fresh conversation (clears memory) |

Everything else you type gets sent to Claude.

## Options

| Setting | Default | What it Does |
|---------|---------|-------------|
| `--account` | (required) | Your account identifier |
| `--model` | opus | Which Claude model to use |
| `--max-budget` | $5.00 | Maximum spend per message |
| `--port` | 8080 | Port for the messenger API (auto-selects if in use) |
| `--api-url` | (auto-managed) | Use an external messenger API instead of auto-managing |

## How Much Does it Cost?

ccchat itself is free. You pay for Claude API usage through your Anthropic subscription. Use `/status` to check your running total, and `--max-budget` to set a per-message spending cap.

## Troubleshooting

**ccchat starts but I don't get replies**
- Check that `claude` works on its own (`claude -p "hello"`)
- Verify your account number is correct

**I sent a message but nothing happened**
- If it's from a new sender, check Note to Self for an approval notification
- Use `/pending` in Note to Self to see blocked senders

**Messages are cut off**
- Long responses are automatically split into multiple messages. They should arrive in order.

## License

MIT
