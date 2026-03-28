package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

// ProviderConf holds credentials for a single LLM provider.
type ProviderConf struct {
	APIKey  string `yaml:"api_key"  json:"api_key"`
	BaseURL string `yaml:"base_url" json:"base_url,omitempty"`
}

// ClawConfig is claw's own config (stored at ~/.claw/config.yaml).
// It deliberately mirrors only the fields claw needs.
type ClawConfig struct {
	Provider  string                  `yaml:"provider"  json:"provider"`
	Model     string                  `yaml:"model"     json:"model"`
	Workspace string                  `yaml:"workspace" json:"workspace"`
	Providers map[string]ProviderConf `yaml:"providers" json:"providers"`
}

// clawHome returns ~/.claw (or $CLAW_HOME).
func clawHome() string {
	if v := os.Getenv("CLAW_HOME"); v != "" {
		return v
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return ".claw"
	}
	return filepath.Join(home, ".claw")
}

// configPath returns the path to claw's config.yaml.
func configPath() string {
	return filepath.Join(clawHome(), "config.yaml")
}

// defaultWorkspace returns ~/.claw/workspace as the default data directory.
func defaultWorkspace() string {
	return filepath.Join(clawHome(), "workspace")
}

// OpenClawConfig 是 OpenClaw JSON 配置格式 (从 ~/.openclaw/openclaw.json 读取)
type OpenClawConfig struct {
	Models  *OpenClawModels  `json:"models,omitempty"`
	Agents  *OpenClawAgents  `json:"agents,omitempty"`
	Bindings []interface{}   `json:"bindings,omitempty"`
}

type OpenClawModels struct {
	Providers map[string]OpenClawProvider `json:"providers,omitempty"`
}

type OpenClawProvider struct {
	BaseURL string `json:"baseUrl,omitempty"`
	APIKey  string `json:"apiKey,omitempty"`
}

type OpenClawAgents struct {
	Defaults *OpenClawDefaults `json:"defaults,omitempty"`
}

type OpenClawDefaults struct {
	Model *OpenClawModel `json:"model,omitempty"`
}

type OpenClawModel struct {
	Primary    string   `json:"primary,omitempty"`
	Fallbacks []string `json:"fallbacks,omitempty"`
}

// openclawHome returns ~/.openclaw
func openclawHome() string {
	home, err := os.UserHomeDir()
	if err != nil {
		return ".openclaw"
	}
	return filepath.Join(home, ".openclaw")
}

// openclawConfigPath returns ~/.openclaw/openclaw.json
func openclawConfigPath() string {
	return filepath.Join(openclawHome(), "openclaw.json")
}

// LoadOpenClawConfig loads OpenClaw config from ~/.openclaw/openclaw.json
func LoadOpenClawConfig() (*OpenClawConfig, error) {
	cfg := &OpenClawConfig{}
	path := openclawConfigPath()

	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, nil // 文件不存在不是错误
		}
		return nil, fmt.Errorf("read openclaw config: %w", err)
	}
	if err := json.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("parse openclaw config: %w", err)
	}
	return cfg, nil
}

// LoadClawConfig loads claw config from ~/.claw/config.yaml.
// Also merges OpenClaw config from ~/.openclaw/openclaw.json if available.
func LoadClawConfig() (*ClawConfig, error) {
	cfg := &ClawConfig{}

	data, err := os.ReadFile(configPath())
	if err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("read claw config: %w", err)
	}
	if err == nil {
		if err := yaml.Unmarshal(data, cfg); err != nil {
			return nil, fmt.Errorf("parse claw config: %w", err)
		}
	}

	// 尝试从 OpenClaw JSON 配置合并
	if openclawCfg, err := LoadOpenClawConfig(); err == nil && openclawCfg != nil {
		cfg = mergeOpenClawConfig(cfg, openclawCfg)
	}

	// Apply defaults.
	if cfg.Provider == "" {
		cfg.Provider = "anthropic"
	}
	if cfg.Model == "" {
		cfg.Model = "claude-sonnet-4-6"
	}
	if cfg.Workspace == "" {
		cfg.Workspace = defaultWorkspace()
	}
	if cfg.Providers == nil {
		cfg.Providers = make(map[string]ProviderConf)
	}

	return cfg, nil
}

// mergeOpenClawConfig 将 OpenClaw JSON 配置合并到 ClawConfig
func mergeOpenClawConfig(cfg *ClawConfig, openclawCfg *OpenClawConfig) *ClawConfig {
	if openclawCfg.Models == nil {
		return cfg
	}

	// 合并 providers
	for name, provider := range openclawCfg.Models.Providers {
		if _, exists := cfg.Providers[name]; !exists {
			// 新增 provider
			cfg.Providers[name] = ProviderConf{
				APIKey:  provider.APIKey,
				BaseURL: provider.BaseURL,
			}
		} else {
			// 更新已存在的 provider（JSON 配置优先）
			existing := cfg.Providers[name]
			if provider.APIKey != "" {
				existing.APIKey = provider.APIKey
			}
			if provider.BaseURL != "" {
				existing.BaseURL = provider.BaseURL
			}
			cfg.Providers[name] = existing
		}
	}

	// 设置主要模型（从 agents.defaults.model.primary）
	if openclawCfg.Agents != nil &&
		openclawCfg.Agents.Defaults != nil &&
		openclawCfg.Agents.Defaults.Model != nil &&
		openclawCfg.Agents.Defaults.Model.Primary != "" {
		cfg.Model = openclawCfg.Agents.Defaults.Model.Primary
		// 从模型名称提取 provider (格式: "provider/model")
		if parts := splitModelName(cfg.Model); len(parts) == 2 {
			cfg.Provider = parts[0]
		}
	}

	return cfg
}

// splitModelName 分割 "provider/model" 格式的模型名
func splitModelName(model string) []string {
	for i := len(model) - 1; i >= 0; i-- {
		if model[i] == '/' {
			return []string{model[:i], model[i+1:]}
		}
	}
	return []string{"", model}
}

// SaveClawConfig persists the config to ~/.claw/config.yaml.
func SaveClawConfig(cfg *ClawConfig) error {
	dir := clawHome()
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return fmt.Errorf("create claw home: %w", err)
	}
	data, err := yaml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("marshal claw config: %w", err)
	}
	return os.WriteFile(configPath(), data, 0o644)
}

// LoadClawConfigPath loads claw config from a specific path.
// Returns an empty config (with defaults) if the file does not exist.
func LoadClawConfigPath(path string) (*ClawConfig, error) {
	cfg := &ClawConfig{}

	data, err := os.ReadFile(path)
	if err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("read claw config: %w", err)
	}
	if err == nil {
		if err := yaml.Unmarshal(data, cfg); err != nil {
			return nil, fmt.Errorf("parse claw config: %w", err)
		}
	}

	// Apply defaults.
	if cfg.Provider == "" {
		cfg.Provider = "anthropic"
	}
	if cfg.Model == "" {
		cfg.Model = "claude-sonnet-4-6"
	}
	if cfg.Workspace == "" {
		cfg.Workspace = defaultWorkspace()
	}
	if cfg.Providers == nil {
		cfg.Providers = make(map[string]ProviderConf)
	}

	return cfg, nil
}

// ActiveAPIKey returns the API key for the currently configured provider.
func (c *ClawConfig) ActiveAPIKey() string {
	if p, ok := c.Providers[c.Provider]; ok {
		return p.APIKey
	}
	return ""
}

// IsConfigured returns true if the active provider has an API key.
func (c *ClawConfig) IsConfigured() bool {
	return c.ActiveAPIKey() != ""
}
