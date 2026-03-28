# Configuration Reference

Config file: `~/.claw/config.yaml`

## Minimal Setup

```yaml
providers:
  anthropic:
    api_key: "sk-..."

provider: anthropic
model: claude-sonnet-4-6
```

Or just: `export ANTHROPIC_API_KEY="sk-..."` and run `claw chat`.

## Full Config

```yaml
providers:
  anthropic:
    api_key: "sk-..."
    base_url: ""                   # optional URL override
    models:                        # optional model metadata
      - id: claude-sonnet-4-6
        reasoning: false
        input: ["text", "image"]
        context_window: 200000
        max_tokens: 8192
        cost:
          input: 3.0
          output: 15.0
          cache_read: 0.3
          cache_write: 3.75
  openai:
    api_key: "sk-..."
    base_url: "https://api.openai.com/v1"
  openai-response:                 # OpenAI-compatible APIs (Perplexity, Together.ai)
    api_key: "sk-..."
    base_url: "https://api.example.com/v1"

channels:
  telegram:
    enabled: true                  # enable/disable this channel (default: true)
    enable_notify: false           # allow notify tool to send to this channel (default: false)
    token: "BOT_TOKEN"
    notify_chat: "123456789"
    channel_id: "@my_channel"
    group_mode: "mention"          # mention | always | disabled
    allowed_ids: [136345060]
  qq:
    enabled: true                  # enable/disable this channel (default: true)
    enable_notify: false           # allow notify tool to send to this channel (default: false)
    app_id: "QQ_BOT_APP_ID"
    app_secret: "QQ_BOT_APP_SECRET"
    group_mode: "mention"
    allowed_ids: []

provider: anthropic
model: claude-sonnet-4-6
model_strong: claude-opus-4-6     # optional tier
model_fast: claude-haiku-4-5      # optional tier
workspace: "~/.claw/workspace"

runner:
  type: go
  system: ""                       # custom system prompt (overrides default)
  idle_timeout: 10                 # minutes before reaping idle runners
  compaction:
    max_tokens: 80000              # auto-compact threshold (-1 = disabled)
    keep_tail: 20                  # recent messages kept after compaction

cron:
  enabled: true
  data_dir: "~/.claw/workspace/cron"
```

## Directory Layout

| Path | Purpose |
|------|---------|
| `~/.claw/config.yaml` | Static config (user-edited) |
| `~/.claw/workspace/state.yaml` | Runtime state: current provider/model (program-managed) |
| `~/.claw/cache/models.json` | Cached model list (safe to delete) |
| `~/.claw/workspace/sessions/` | Chat session history |
| `~/.claw/workspace/memory/` | Persistent memory (SOUL.md, USER.md, FACT.md, JOURNAL.jsonl) |
| `~/.claw/workspace/skills/` | Installed skills |
| `~/.claw/workspace/cron/` | Cron job persistence |

## Environment Variables

Priority (highest wins): env vars > state.yaml > config.yaml > defaults.

| Variable | Overrides |
|----------|-----------|
| `CLAW_HOME` | claw home directory (default `~/.claw`) |
| `CLAW_PROVIDER` | `provider` |
| `CLAW_MODEL` | `model` |
| `CLAW_MODEL_STRONG` | `model_strong` |
| `CLAW_MODEL_FAST` | `model_fast` |
| `CLAW_WORKSPACE` | `workspace` |
| `CLAW_RUNNER_IDLE_TIMEOUT` | `runner.idle_timeout` |
| `CLAW_CRON_ENABLED` | `cron.enabled` |
| `CLAW_TELEGRAM_ENABLED` | `channels.telegram.enabled` |
| `CLAW_TELEGRAM_ENABLE_NOTIFY` | `channels.telegram.enable_notify` |
| `CLAW_TELEGRAM_TOKEN` | `channels.telegram.token` |
| `CLAW_TELEGRAM_NOTIFY_CHAT` | `channels.telegram.notify_chat` |
| `CLAW_TELEGRAM_GROUP_MODE` | `channels.telegram.group_mode` |
| `CLAW_TELEGRAM_ALLOWED_IDS` | `channels.telegram.allowed_ids` (comma-separated) |
| `CLAW_QQ_ENABLED` | `channels.qq.enabled` |
| `CLAW_QQ_ENABLE_NOTIFY` | `channels.qq.enable_notify` |
| `CLAW_QQ_APP_ID` | `channels.qq.app_id` |
| `CLAW_QQ_APP_SECRET` | `channels.qq.app_secret` |
| `ANTHROPIC_API_KEY` | `providers.anthropic.api_key` |
| `ANTHROPIC_BASE_URL` | `providers.anthropic.base_url` |
| `OPENAI_API_KEY` | `providers.openai.api_key` (also used by `openai-response`) |
| `OPENAI_BASE_URL` | `providers.openai.base_url` (also used by `openai-response`) |

## Defaults

| Field | Default |
|-------|---------|
| `provider` | `anthropic` |
| `model` | `claude-sonnet-4-6` |
| `workspace` | `~/.claw/workspace` |
| `runner.type` | `go` |
| `runner.idle_timeout` | `10` (minutes) |
| `runner.compaction.max_tokens` | `80000` |
| `runner.compaction.keep_tail` | `20` |
| `cron.enabled` | `true` |
| `channels.telegram.enabled` | `true` |
| `channels.telegram.enable_notify` | `false` |
| `channels.telegram.group_mode` | `mention` |
| `channels.qq.enabled` | `true` |
| `channels.qq.enable_notify` | `false` |
| `channels.qq.group_mode` | `mention` |
