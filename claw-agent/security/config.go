package security

import (
	"encoding/json"
	"log/slog"
	"os"
	"path/filepath"
	"sync"
)

// SecurityConfig 用户自定义安全配置
type SecurityConfig struct {
	AllowedPaths []string `json:"allowedPaths"` // 用户追加的信任目录
}

var (
	configMu   sync.RWMutex
	cachedConfig *SecurityConfig
)

// LoadSecurityConfig 从 ~/.openclaw/security.json 加载安全配置
func LoadSecurityConfig(openclawHome string) *SecurityConfig {
	configMu.RLock()
	if cachedConfig != nil {
		defer configMu.RUnlock()
		return cachedConfig
	}
	configMu.RUnlock()

	configPath := filepath.Join(openclawHome, "security.json")

	data, err := os.ReadFile(configPath)
	if err != nil {
		if os.IsNotExist(err) {
			slog.Debug("security config not found, using defaults", "path", configPath)
			return &SecurityConfig{}
		}
		slog.Warn("failed to read security config", "path", configPath, "error", err)
		return &SecurityConfig{}
	}

	var cfg SecurityConfig
	if err := json.Unmarshal(data, &cfg); err != nil {
		slog.Warn("failed to parse security config", "path", configPath, "error", err)
		return &SecurityConfig{}
	}

	configMu.Lock()
	cachedConfig = &cfg
	configMu.Unlock()

	slog.Info("loaded security config", "allowed_paths", len(cfg.AllowedPaths))
	return &cfg
}

// SaveSecurityConfig 保存安全配置到 ~/.openclaw/security.json
func SaveSecurityConfig(openclawHome string, cfg *SecurityConfig) error {
	configPath := filepath.Join(openclawHome, "security.json")

	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return err
	}

	if err := os.WriteFile(configPath, data, 0644); err != nil {
		return err
	}

	configMu.Lock()
	cachedConfig = cfg
	configMu.Unlock()

	slog.Info("saved security config", "path", configPath)
	return nil
}

// MergeToPolicy 将用户自定义路径合并到 PathPolicy
func (c *SecurityConfig) MergeToPolicy(base PathPolicy) PathPolicy {
	if len(c.AllowedPaths) == 0 {
		return base
	}

	merged := PathPolicy{
		AllowedPrefixes: make([]string, 0, len(base.AllowedPrefixes)+len(c.AllowedPaths)),
		DeniedPrefixes:  base.DeniedPrefixes,
	}

	// 保留原有路径
	merged.AllowedPrefixes = append(merged.AllowedPrefixes, base.AllowedPrefixes...)

	// 追加用户自定义路径（去重）
	existing := make(map[string]bool, len(base.AllowedPrefixes))
	for _, p := range base.AllowedPrefixes {
		existing[p] = true
	}

	for _, p := range c.AllowedPaths {
		if !existing[p] {
			merged.AllowedPrefixes = append(merged.AllowedPrefixes, p)
			existing[p] = true
		}
	}

	return merged
}
