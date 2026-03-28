package tool

import (
	"context"
	"encoding/json"
	"fmt"

	aitypes "claw-agent/ai/types"
	"claw-agent/tools/openclaw"
)

// OpenClawConfigTool OpenClaw 配置工具
type OpenClawConfigTool struct {
	configTool *openclaw.Tool
}

// NewOpenClawConfigTool 创建新的配置工具
func NewOpenClawConfigTool() *OpenClawConfigTool {
	return &OpenClawConfigTool{
		configTool: openclaw.New(),
	}
}

// Definition 返回工具定义
func (t *OpenClawConfigTool) Definition() aitypes.ToolDefinition {
	return aitypes.ToolDefinition{
		Name:        "openclaw_config",
		Description: "Manage OpenClaw configuration: read/write config, add/remove providers, set models",
		InputSchema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"action": map[string]interface{}{
					"type":        "string",
					"enum":        []string{"read", "write", "add_provider", "remove_provider", "set_model", "add_fallback", "validate"},
					"description": "The action to perform",
				},
				"data": map[string]interface{}{
					"type":        "object",
					"description": "Action-specific data",
				},
			},
			"required": []string{"action"},
		},
	}
}

// Execute 执行工具
func (t *OpenClawConfigTool) Execute(ctx context.Context, args map[string]any) (string, error) {
	action, ok := args["action"].(string)
	if !ok {
		return "", fmt.Errorf("action is required")
	}

	data, _ := args["data"].(map[string]any)

	switch action {
	case "read":
		return t.readConfig()
	case "validate":
		return t.validateConfig()
	case "add_provider":
		return t.addProvider(data)
	case "remove_provider":
		return t.removeProvider(data)
	case "set_model":
		return t.setModel(data)
	case "add_fallback":
		return t.addFallback(data)
	default:
		return "", fmt.Errorf("unknown action: %s", action)
	}
}

// readConfig 读取配置
func (t *OpenClawConfigTool) readConfig() (string, error) {
	cfg, err := t.configTool.ReadConfig()
	if err != nil {
		return "", fmt.Errorf("read config failed: %w", err)
	}

	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return "", fmt.Errorf("marshal config: %w", err)
	}

	return string(data), nil
}

// validateConfig 验证配置
func (t *OpenClawConfigTool) validateConfig() (string, error) {
	cfg, err := t.configTool.ReadConfig()
	if err != nil {
		return "", fmt.Errorf("read config failed: %w", err)
	}

	if err := t.configTool.ValidateConfig(cfg); err != nil {
		return "", fmt.Errorf("validation failed: %w", err)
	}

	return "Configuration is valid", nil
}

// addProvider 添加服务商
func (t *OpenClawConfigTool) addProvider(data map[string]any) (string, error) {
	name, _ := data["name"].(string)
	baseURL, _ := data["base_url"].(string)
	apiKey, _ := data["api_key"].(string)

	if name == "" || baseURL == "" {
		return "", fmt.Errorf("name and base_url are required")
	}

	if err := t.configTool.AddProvider(name, baseURL, apiKey); err != nil {
		return "", fmt.Errorf("add provider failed: %w", err)
	}

	return fmt.Sprintf("Provider '%s' added successfully", name), nil
}

// removeProvider 移除服务商
func (t *OpenClawConfigTool) removeProvider(data map[string]any) (string, error) {
	name, _ := data["name"].(string)

	if name == "" {
		return "", fmt.Errorf("name is required")
	}

	if err := t.configTool.RemoveProvider(name); err != nil {
		return "", fmt.Errorf("remove provider failed: %w", err)
	}

	return fmt.Sprintf("Provider '%s' removed successfully", name), nil
}

// setModel 设置主要模型
func (t *OpenClawConfigTool) setModel(data map[string]any) (string, error) {
	model, _ := data["model"].(string)

	if model == "" {
		return "", fmt.Errorf("model is required")
	}

	if err := t.configTool.SetModel(model); err != nil {
		return "", fmt.Errorf("set model failed: %w", err)
	}

	return fmt.Sprintf("Model set to '%s'", model), nil
}

// addFallback 添加备用模型
func (t *OpenClawConfigTool) addFallback(data map[string]any) (string, error) {
	model, _ := data["model"].(string)

	if model == "" {
		return "", fmt.Errorf("model is required")
	}

	if err := t.configTool.AddFallbackModel(model); err != nil {
		return "", fmt.Errorf("add fallback model failed: %w", err)
	}

	return fmt.Sprintf("Fallback model '%s' added successfully", model), nil
}
