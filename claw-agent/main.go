package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"claw-agent/agent"
	"claw-agent/agent/runner"
	"claw-agent/agent/store"
	"claw-agent/agent/tool"
	"claw-agent/memory"
	"claw-agent/security"
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

	// 安全参数
	authToken := flag.String("auth-token", "", "WebSocket authentication token")

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

		if err := runAgentMode(*port, *configPath, llmConfig, *authToken); err != nil {
			log.Fatalf("Failed to run agent mode: %v", err)
		}
	} else {
		log.Fatal("Agent mode is required. Use --agent flag to start.")
	}
}

// runAgentMode 启动 Agent 模式
func runAgentMode(port int, configPath string, llmConfig *LLMConfig, authToken string) error {
	log.Printf("Starting Claw Agent v%s (commit %s)", version, commit)

	// 创建根上下文，用于控制生命周期
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// 设置信号处理：捕获 SIGINT 和 SIGTERM
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	// 1. 确定最终配置
	cfg, err := resolveConfig(configPath, llmConfig)
	if err != nil {
		return fmt.Errorf("resolve config: %w", err)
	}

	// 2. 初始化 Agent Pool（传入上下文用于 Pool Reaper）
	broker := security.NewApprovalBroker()

	// 加载用户安全配置（路径白名单）
	securityCfg := security.LoadSecurityConfig(openclawHome())
	if len(securityCfg.AllowedPaths) > 0 {
		log.Printf("Loaded %d custom allowed paths from security config", len(securityCfg.AllowedPaths))
	}

	pool, err := initAgentPool(ctx, cfg, broker, securityCfg)
	if err != nil {
		return fmt.Errorf("init agent pool: %w", err)
	}
	defer pool.Close()

	// 3. 启动 WebSocket 服务器（传入可选的认证 token 和审批 broker）
	srv := server.New(pool, port, ctx, authToken, broker)
	if err := srv.Start(); err != nil {
		return fmt.Errorf("start server: %w", err)
	}

	log.Printf("Claw Agent started successfully on port %d", srv.Port())
	log.Printf("Using provider: %s, model: %s", cfg.Provider, cfg.Model)

	// 4. 等待信号或服务器错误
	select {
	case sig := <-sigCh:
		log.Printf("Received signal %v, initiating graceful shutdown...", sig)
		// 触发上下文取消
		cancel()
	case <-ctx.Done():
		// 上下文已被取消
	}

	// 5. 优雅关闭（10秒超时）
	if err := srv.Shutdown(10 * time.Second); err != nil {
		log.Printf("Server shutdown error (may be expected): %v", err)
	}

	log.Printf("Claw Agent shutdown complete")
	return nil
}

// resolveConfig 确定最终配置
// 优先级：命令行参数 > 配置文件
func resolveConfig(configPath string, llmConfig *LLMConfig) (*ClawConfig, error) {
	// 如果命令行指定了 API Key，优先使用命令行配置
	if llmConfig.API != "" && llmConfig.APIKey != "" {
		log.Printf("Using LLM config from command line arguments")

		workDir := llmConfig.WorkDir
		if workDir == "" {
			workDir = DefaultWorkspace()
		}

		// Ensure workspace directory exists
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
func initAgentPool(ctx context.Context, cfg *ClawConfig, broker *security.ApprovalBroker, securityCfg *security.SecurityConfig) (*agent.Pool, error) {
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

	// Register OpenClaw config tool
	openclawTool := tool.NewOpenClawConfigTool()

	sessDir := cfg.Workspace + "/sessions"
	if err := os.MkdirAll(sessDir, 0755); err != nil {
		return nil, fmt.Errorf("create sessions dir: %w", err)
	}

	fs, err := store.NewFileStore(sessDir, cfg.Workspace)
	if err != nil {
		return nil, fmt.Errorf("create file store: %w", err)
	}

	// Create Memory Store (soul/user/fact)
	memoryStore := memory.NewStore(cfg.Workspace)

	factory := func(ctx context.Context, model string) (runner.Runner, error) {
		if model == "" {
			model = cfg.Model
		}
		return runner.NewGoRunner(ctx, runner.GoRunnerConfig{
			API:            cfg.Provider,
			Model:          model,
			APIKey:         pc.APIKey,
			BaseURL:        pc.BaseURL,
			WorkDir:        cfg.Workspace,
			Workspace:      cfg.Workspace,
			AnnaHome:       clawHome(),
			MemoryStore:    memoryStore,
			ExtraTools:     []tool.Tool{openclawTool},
			Broker:         broker,
			SecurityConfig: securityCfg,
		})
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

	// Start Pool Reaper for idle runner cleanup
	reaperCtx := ctx
	if reaperCtx == nil {
		reaperCtx = context.Background()
	}
	go pool.StartReaper(reaperCtx)

	return pool, nil
}
