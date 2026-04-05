package openclaw

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"claw-agent/openclawtypes"
)

const (
	defaultConfigPath = "~/.openclaw/openclaw.json"
	backupDir         = "~/.openclaw/backups"
)

// Type aliases for backward compatibility
type Config = openclawtypes.OpenClawConfig
type ModelsConfig = openclawtypes.OpenClawModels
type ProviderConfig = openclawtypes.OpenClawProvider

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
	if cfg.Models == nil || cfg.Models.Providers == nil {
		return fmt.Errorf("models configuration is required")
	}
	if cfg.Agents != nil && cfg.Agents.Defaults != nil && cfg.Agents.Defaults.Model != nil {
		if cfg.Agents.Defaults.Model.Primary != "" {
			provider, _ := splitModelID(cfg.Agents.Defaults.Model.Primary)
			if provider != "" {
				if _, exists := cfg.Models.Providers[provider]; !exists {
					return fmt.Errorf("provider '%s' for primary model not configured", provider)
				}
			}
		}
		// 验证备用模型
		for _, fallbackModel := range cfg.Agents.Defaults.Model.Fallbacks {
			p, _ := splitModelID(fallbackModel)
			if p == "" {
				return fmt.Errorf("invalid fallback model format: %s", fallbackModel)
			}
			if _, exists := cfg.Models.Providers[p]; !exists {
				return fmt.Errorf("fallback provider '%s' not configured", p)
			}
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
			Models: &ModelsConfig{
				Providers: make(map[string]ProviderConfig),
			},
			Agents: &openclawtypes.OpenClawAgents{
				Defaults: &openclawtypes.OpenClawDefaults{
					Model: &openclawtypes.OpenClawModel{
						Primary: "openai/gpt-4o",
					},
				},
			},
		}
	}

	// 确保 Models 存在
	if cfg.Models == nil {
		cfg.Models = &ModelsConfig{
			Providers: make(map[string]ProviderConfig),
		}
	}
	if cfg.Models.Providers == nil {
		cfg.Models.Providers = make(map[string]ProviderConfig)
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

	// 确保嵌套结构存在
	if cfg.Agents == nil {
		cfg.Agents = &openclawtypes.OpenClawAgents{}
	}
	if cfg.Agents.Defaults == nil {
		cfg.Agents.Defaults = &openclawtypes.OpenClawDefaults{}
	}
	if cfg.Agents.Defaults.Model == nil {
		cfg.Agents.Defaults.Model = &openclawtypes.OpenClawModel{}
	}

	cfg.Agents.Defaults.Model.Primary = model
	return t.WriteConfig(cfg)
}

// AddFallbackModel 添加备用模型
func (t *Tool) AddFallbackModel(model string) error {
	cfg, err := t.ReadConfig()
	if err != nil {
		return err
	}

	// 确保嵌套结构存在
	if cfg.Agents == nil {
		cfg.Agents = &openclawtypes.OpenClawAgents{}
	}
	if cfg.Agents.Defaults == nil {
		cfg.Agents.Defaults = &openclawtypes.OpenClawDefaults{}
	}
	if cfg.Agents.Defaults.Model == nil {
		cfg.Agents.Defaults.Model = &openclawtypes.OpenClawModel{}
	}

	for _, m := range cfg.Agents.Defaults.Model.Fallbacks {
		if m == model {
			return fmt.Errorf("model already exists in fallback list")
		}
	}

	cfg.Agents.Defaults.Model.Fallbacks = append(cfg.Agents.Defaults.Model.Fallbacks, model)
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

	// 生成备份文件名（使用当前时间戳）
	backupFile := filepath.Join(backupPath, "openclaw-"+time.Now().Format("20060102-150405")+".json")

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
