# Model Management

## Tiered Models

Two tiers for different workloads, each falls back to `model` when not set:

| Tier | Config Field | Use Case |
|------|-------------|----------|
| strong | `model_strong` | Heavy reasoning, complex tasks |
| fast | `model_fast` | Quick responses, simple queries |

```yaml
model: claude-sonnet-4-6          # default
model_strong: claude-opus-4-6     # optional
model_fast: claude-haiku-4-5      # optional
```

## CLI Commands

```bash
claw models             # List available models
claw models list        # List all models grouped by provider
claw models update      # Fetch from provider APIs, update cache
claw models current     # Show active provider/model
claw models set <p/m>   # Switch (e.g. claw models set openai/gpt-4o)
claw models search <q>  # Search by name
```

The cache at `~/.claw/cache/models.json` is populated by `claw models update`. Without it, only models in config are shown.

## Provider Setup

### Anthropic

```yaml
providers:
  anthropic:
    api_key: "sk-..."
```

Or: `export ANTHROPIC_API_KEY="sk-..."`

### OpenAI

```yaml
providers:
  openai:
    api_key: "sk-..."
    base_url: "https://api.openai.com/v1"  # optional
```

Or: `export OPENAI_API_KEY="sk-..."`

### OpenAI-Compatible (Responses API)

For Perplexity, Together.ai, or any OpenAI-compatible service:

```yaml
providers:
  openai-response:
    api_key: "sk-..."
    base_url: "https://api.perplexity.ai"
```

Uses same `OPENAI_API_KEY` / `OPENAI_BASE_URL` env vars.

## Runtime Switching

- **CLI**: `/model` in-chat command
- **Telegram**: inline keyboard model picker
- **Persistent**: `claw models set provider/model` writes to state.yaml
