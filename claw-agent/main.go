package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"time"

	"claw-agent/agent"
	"claw-agent/agent/runner"
	"claw-agent/agent/store"
	"claw-agent/server"
)

var (
	version = "2.1.0"
	commit  = "unknown"
)

// LLMConfig 命令行传入的 LLM 配置
type LLMConfig struct {
	API     string
	Model   string
	APIKey  string
	BaseURL string
	WorkDir string
}

func main() {
	// 命令行参数
	agentMode := flag.Bool("agent", false, "Run as agent mode")
	port := flag.Int("port", 0, "WebSocket server port (0 for random available port)")
	configPath := flag.String("config", "", "Path to config file")
	showVersion := flag.Bool("version", false, "Show version information")

	// LLM 配置参数
	llmAPI := flag.String("api", "", "LLM API type (anthropic, openai, openai-response)")
	llmModel := flag.String("model", "", "LLM model name")
	llmAPIKey := flag.String("api-key", "", "LLM API key")
	llmBaseURL := flag.String("base-url", "", "LLM API base URL")
	llmWorkDir := flag.String("workspace", "", "Workspace directory")

	flag.Parse()

	if *showVersion {
		fmt.Printf("Claw Agent v%s (commit %s)\n", version, commit)
		os.Exit(0)
	}

	if *agentMode {
		// 构建 LLM 配置
		llmConfig := &LLMConfig{
			API:     *llmAPI,
			Model:   *llmModel,
			APIKey:  *llmAPIKey,
			BaseURL: *llmBaseURL,
			WorkDir: *llmWorkDir,
		}

		if err := runAgentMode(*port, *configPath, llmConfig); err != nil {
			log.Fatalf("Failed to run agent mode: %v", err)
		}
	} else {
		log.Fatal("Agent mode is required. Use --agent flag to start.")
	}
}

// runAgentMode 启动 Agent 模式
func runAgentMode(port int, configPath string, llmConfig *LLMConfig) error {
	log.Printf("Starting Claw Agent v%s (commit %s)", version, commit)

	// 1. 确定最终配置
	cfg, err := resolveConfig(configPath, llmConfig)
	if err != nil {
		return fmt.Errorf("resolve config: %w", err)
	}

	// 2. 初始化 Agent Pool
	pool, err := initAgentPool(cfg)
	if err != nil {
		return fmt.Errorf("init agent pool: %w", err)
	}
	defer pool.Close()

	// 3. 启动 WebSocket 服务器
	srv := server.New(pool, port)
	if err := srv.Start(); err != nil {
		return fmt.Errorf("start server: %w", err)
	}

	log.Printf("Claw Agent started successfully on port %d", srv.Port())
	log.Printf("Using provider: %s, model: %s", cfg.Provider, cfg.Model)

	// 4. 阻塞等待
	select {}
}

// resolveConfig 确定最终配置
// 优先级：命令行参数 > 配置文件
func resolveConfig(configPath string, llmConfig *LLMConfig) (*ClawConfig, error) {
	// 如果命令行指定了 API Key，优先使用命令行配置
	if llmConfig.API != "" && llmConfig.APIKey != "" {
		log.Printf("Using LLM config from command line arguments")

		workDir := llmConfig.WorkDir
		if workDir == "" {
			home, _ := os.UserHomeDir()
			workDir = home + "/.openclawswitch/workspace"
		}

		// 确保 workspace 目录存在
		if err := os.MkdirAll(workDir, 0755); err != nil {
			return nil, fmt.Errorf("create workspace: %w", err)
		}

		cfg := &ClawConfig{
			Provider:  llmConfig.API,
			Model:     llmConfig.Model,
			Workspace: workDir,
			Providers: map[string]ProviderConf{
				llmConfig.API: {
					APIKey:  llmConfig.APIKey,
					BaseURL: llmConfig.BaseURL,
				},
			},
		}

		// 设置默认值
		if cfg.Model == "" {
			// 根据 API 类型设置默认模型
			switch cfg.Provider {
			case "anthropic":
				cfg.Model = "claude-sonnet-4-6"
			case "openai", "openai-response":
				cfg.Model = "gpt-4o"
			}
		}

		return cfg, nil
	}

	// 否则从配置文件加载
	log.Printf("Loading config from file")
	if configPath != "" {
		return LoadClawConfigPath(configPath)
	}
	return LoadClawConfig()
}

// initAgentPool 初始化 Agent Pool
func initAgentPool(cfg *ClawConfig) (*agent.Pool, error) {
	// 验证配置
	if cfg.Provider == "" {
		return nil, fmt.Errorf("provider is required")
	}
	if cfg.Model == "" {
		return nil, fmt.Errorf("model is required")
	}

	pc, ok := cfg.Providers[cfg.Provider]
	if !ok {
		return nil, fmt.Errorf("provider '%s' not found in config", cfg.Provider)
	}
	if pc.APIKey == "" {
		return nil, fmt.Errorf("API key for provider '%s' is empty", cfg.Provider)
	}

	factory := func(ctx context.Context, model string) (runner.Runner, error) {
		if model == "" {
			model = cfg.Model
		}
		return runner.NewGoRunner(ctx, runner.GoRunnerConfig{
			API:       cfg.Provider,
			Model:     model,
			APIKey:    pc.APIKey,
			BaseURL:   pc.BaseURL,
			WorkDir:   cfg.Workspace,
			Workspace: cfg.Workspace,
		})
	}

	sessDir := cfg.Workspace + "/sessions"
	if err := os.MkdirAll(sessDir, 0755); err != nil {
		return nil, fmt.Errorf("create sessions dir: %w", err)
	}

	fs, err := store.NewFileStore(sessDir, cfg.Workspace)
	if err != nil {
		return nil, fmt.Errorf("create file store: %w", err)
	}

	compaction := agent.CompactionConfig{
		MaxTokens: 80000,
		KeepTail:  20,
	}

	pool := agent.NewPool(factory,
		agent.WithStore(fs),
		agent.WithDefaultModel(cfg.Model),
		agent.WithIdleTimeout(10*time.Minute),
		agent.WithCompaction(compaction),
	)

	return pool, nil
}
