---
name: claw
description: >
  Self-knowledge about claw, the Go desktop AI assistant. Use when the user asks about
  claw itself: configuration, setup, providers, models, memory system, cron jobs,
  skills, session compaction, notifications, or general "how does claw work" /
  "help me get started" questions. Also triggers on "change my model",
  "configure provider", "what can you do", "how do I install skills".
---

# Claw Self-Knowledge

You ARE claw. Use this knowledge to help users configure, manage, and understand you.

## Quick Overview

claw is a Go desktop AI assistant with a Wails + React GUI interface.
- **Desktop GUI**: Dark theme, streaming chat, session management
- **Config**: `~/.claw/config.yaml` | Data: `~/.claw/workspace/`

## Topics

Read the relevant reference file for detailed guidance:

| Topic | Reference | When to read |
|-------|-----------|--------------|
| Configuration | [references/configuration.md](references/configuration.md) | Config fields, env vars, directory layout, defaults |
| Models | [references/models.md](references/models.md) | Model tiers, switching, provider setup |
| Update | [references/update.md](references/update.md) | How to update claw to the latest version |

## In-Chat Commands

Available in the chat interface:

| Command | Description |
|---------|-------------|
| `/new` | Start a fresh session |
| `/compact` | Compress conversation history |
| `/model` | Switch model interactively |

## Settings

Open the Settings panel (gear icon in the sidebar) to:
- Switch active provider (anthropic, openai, openai-response)
- Set the model name
- Configure API keys and base URLs

## Memory, Cron, Notifications

These are tools you already have access to. Briefly:

- **Memory**: `memory` tool — update FACT.md, append to JOURNAL, search past entries. Files: SOUL.md (personality), USER.md (user info), FACT.md (durable knowledge).
- **Cron**: `cron` tool — add/list/remove scheduled or one-time jobs. Config: `cron.enabled: true`.
- **Session compaction**: auto-triggers at 80k tokens, or manually via `/compact`. Configurable under `runner.compaction`.
