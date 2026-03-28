package openclaw

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

const (
	defaultConfigPath = "~/.openclaw/openclaw.json"
	backupDir         = "~/.openclaw/backups"
)

// Config OpenClaw 配置结构
type Config struct {
	Models ModelsConfig `json:"models"`
	Agent  AgentConfig  `json:"agent"`
}

// ModelsConfig 模型配置
type ModelsConfig struct {
	Providers map[string]ProviderConfig `json:"providers"`
}

// ProviderConfig 服务商配置
type ProviderConfig struct {
	BaseURL string `json:"baseUrl"`
	APIKey  string `json:"apiKey,omitempty"`
}

// AgentConfig Agent 配置
type AgentConfig struct {
	Model           string   `json:"model"`
	FallbackModels  []string `json:"fallbackModels,omitempty"`
	SystemPrompt    string   `json:"systemPrompt,omitempty"`
	Temperature     float64  `json:"temperature,omitempty"`
	MaxTokens       int      `json:"maxTokens,omitempty"`
}

// Tool OpenClaw 配置工具
type Tool struct {
	configPath string
}

// New 创建新的配置工具
func New() *Tool {
	return &Tool{
		configPath: expandPath(defaultConfigPath),
	}
}

// NewWithPath 创建指定路径的配置工具
func NewWithPath(path string) *Tool {
	return &Tool{
		configPath: expandPath(path),
	}
}

// ReadConfig 读取配置
func (t *Tool) ReadConfig() (*Config, error) {
	data, err := os.ReadFile(t.configPath)
	if err != nil {
		return nil, fmt.Errorf("read config file: %w", err)
	}

	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("parse config: %w", err)
	}

	return &cfg, nil
}

// WriteConfig 写入配置
func (t *Tool) WriteConfig(cfg *Config) error {
	// 创建备份
	if err := t.backupConfig(); err != nil {
		return fmt.Errorf("backup config: %w", err)
	}

	// 确保目录存在
	dir := filepath.Dir(t.configPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("create config dir: %w", err)
	}

	// 格式化 JSON
	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return fmt.Errorf("marshal config: %w", err)
	}

	// 写入文件
	if err := os.WriteFile(t.configPath, data, 0644); err != nil {
		return fmt.Errorf("write config: %w", err)
	}

	return nil
}

// ValidateConfig 验证配置
func (t *Tool) ValidateConfig(cfg *Config) error {
	if cfg.Agent.Model == "" {
		return fmt.Errorf("agent model is required")
	}

	// 检查服务商是否存在
	provider, _ := splitModelID(cfg.Agent.Model)
	if provider == "" {
		return fmt.Errorf("invalid model format: %s", cfg.Agent.Model)
	}

	if _, exists := cfg.Models.Providers[provider]; !exists {
		return fmt.Errorf("provider '%s' not configured", provider)
	}

	// 验证备用模型
	for _, fallbackModel := range cfg.Agent.FallbackModels {
		p, _ := splitModelID(fallbackModel)
		if p == "" {
			return fmt.Errorf("invalid fallback model format: %s", fallbackModel)
		}
		if _, exists := cfg.Models.Providers[p]; !exists {
			return fmt.Errorf("fallback provider '%s' not configured", p)
		}
	}

	return nil
}

// AddProvider 添加服务商
func (t *Tool) AddProvider(name, baseURL, apiKey string) error {
	cfg, err := t.ReadConfig()
	if err != nil {
		// 如果配置不存在，创建新配置
		cfg = &Config{
			Models: ModelsConfig{
				Providers: make(map[string]ProviderConfig),
			},
			Agent: AgentConfig{
				Model: "openai/gpt-4o",
			},
		}
	}

	// 添加服务商
	cfg.Models.Providers[name] = ProviderConfig{
		BaseURL: baseURL,
		APIKey:  apiKey,
	}

	return t.WriteConfig(cfg)
}

// RemoveProvider 移除服务商
func (t *Tool) RemoveProvider(name string) error {
	cfg, err := t.ReadConfig()
	if err != nil {
		return err
	}

	if _, exists := cfg.Models.Providers[name]; !exists {
		return fmt.Errorf("provider '%s' not found", name)
	}

	delete(cfg.Models.Providers, name)
	return t.WriteConfig(cfg)
}

// SetModel 设置主要模型
func (t *Tool) SetModel(model string) error {
	cfg, err := t.ReadConfig()
	if err != nil {
		return err
	}

	cfg.Agent.Model = model
	return t.WriteConfig(cfg)
}

// AddFallbackModel 添加备用模型
func (t *Tool) AddFallbackModel(model string) error {
	cfg, err := t.ReadConfig()
	if err != nil {
		return err
	}

	for _, m := range cfg.Agent.FallbackModels {
		if m == model {
			return fmt.Errorf("model already exists in fallback list")
		}
	}

	cfg.Agent.FallbackModels = append(cfg.Agent.FallbackModels, model)
	return t.WriteConfig(cfg)
}

// backupConfig 备份配置
func (t *Tool) backupConfig() error {
	// 检查配置文件是否存在
	if _, err := os.Stat(t.configPath); os.IsNotExist(err) {
		return nil // 不存在，无需备份
	}

	// 创建备份目录
	backupPath := expandPath(backupDir)
	if err := os.MkdirAll(backupPath, 0755); err != nil {
		return err
	}

	// 生成备份文件名
	timestamp := "20060102-150405"
	backupFile := filepath.Join(backupPath, "openclaw-"+timestamp+".json")

	// 复制文件
	data, err := os.ReadFile(t.configPath)
	if err != nil {
		return err
	}

	return os.WriteFile(backupFile, data, 0644)
}

// splitModelID 分割模型 ID
func splitModelID(modelID string) (provider, model string) {
	for i, c := range modelID {
		if c == '/' {
			return modelID[:i], modelID[i+1:]
		}
	}
	return "", modelID
}

// expandPath 展开路径中的 ~
func expandPath(path string) string {
	if len(path) > 0 && path[0] == '~' {
		home, _ := os.UserHomeDir()
		return filepath.Join(home, path[1:])
	}
	return path
}
