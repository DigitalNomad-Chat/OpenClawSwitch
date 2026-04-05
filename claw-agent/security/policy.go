package security

import (
	"log/slog"
)

// Decision 表示安全策略的决策结果
type Decision int

const (
	AutoAllow Decision = iota // 自动放行
	AskUser                   // 需要用户确认
	AutoDeny                  // 自动拒绝
)

// SecurityPolicy 综合路径策略和审批 broker 的安全策略引擎
type SecurityPolicy struct {
	PathPolicy PathPolicy
	broker     *ApprovalBroker
	log        *slog.Logger
}

// NewSecurityPolicy 创建默认安全策略
func NewSecurityPolicy(broker *ApprovalBroker, workspace string) *SecurityPolicy {
	return &SecurityPolicy{
		PathPolicy: DefaultPolicy(workspace),
		broker:     broker,
		log:        slog.With("component", "security_policy"),
	}
}

// Evaluate 根据工具名称和参数评估安全决策
func (p *SecurityPolicy) Evaluate(toolName string, args map[string]any) Decision {
	switch toolName {
	case "webfetch":
		// 只读网络请求，无文件系统访问
		return AutoAllow

	case "read":
		path, _ := args["file_path"].(string)
		if path == "" {
			return AskUser
		}
		allowed, _ := p.PathPolicy.Check(path)
		if allowed {
			return AutoAllow
		}
		return AskUser

	case "bash":
		// 任何 bash 命令都需要用户确认
		return AskUser

	case "write":
		// 写文件操作需要用户确认
		return AskUser

	case "edit":
		// 编辑文件操作需要用户确认
		return AskUser

	case "openclaw_config":
		action, _ := args["action"].(string)
		switch action {
		case "read", "validate":
			return AutoAllow
		case "write", "remove", "set", "add":
			return AskUser
		default:
			return AskUser
		}

	default:
		// 未知工具默认需要确认
		return AskUser
	}
}

// RiskLevel 返回工具操作的风险等级
func (p *SecurityPolicy) RiskLevel(toolName string, args map[string]any) string {
	decision := p.Evaluate(toolName, args)
	switch decision {
	case AutoAllow:
		return "low"
	case AutoDeny:
		return "high"
	case AskUser:
		switch toolName {
		case "bash":
			return "high"
		case "write", "edit":
			return "medium"
		case "read":
			return "low"
		default:
			return "medium"
		}
	}
	return "medium"
}
