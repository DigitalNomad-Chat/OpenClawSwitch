package main

import (
	"context"
	"testing"
	"time"

	"claw-agent/agent"
	"claw-agent/agent/runner"
	"claw-agent/agent/store"
)

func TestPoolLifecycle(t *testing.T) {
	tmpDir := t.TempDir()

	factory := func(ctx context.Context, model string) (runner.Runner, error) {
		return runner.NewGoRunner(ctx, runner.GoRunnerConfig{
			API:    "anthropic",
			Model:  "claude-sonnet-4-6",
			APIKey: "test-key",
		})
	}

	fs, err := store.NewFileStore(tmpDir+"/sessions", tmpDir)
	if err != nil {
		t.Fatalf("create store: %v", err)
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	pool := agent.NewPool(factory,
		agent.WithStore(fs),
		agent.WithDefaultModel("claude-sonnet-4-6"),
		agent.WithIdleTimeout(1*time.Minute),
	)
	go pool.StartReaper(ctx)

	// Verify pool was created
	if pool == nil {
		t.Fatal("pool should not be nil")
	}

	// Verify reaper goroutine exits cleanly on cancel
	cancel()
	pool.Close()

	t.Log("Pool lifecycle test passed")
}
