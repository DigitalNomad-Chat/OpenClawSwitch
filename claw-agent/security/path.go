package security

import (
	"os"
	"path/filepath"
	"strings"
)

// PathPolicy 定义文件系统路径访问策略
type PathPolicy struct {
	AllowedPrefixes []string // 允许的路径前缀列表
	DeniedPrefixes  []string // 拒绝的路径前缀列表（优先级高于 Allowed）
}

// DefaultPolicy 返回基于 workspace 的默认安全路径策略
// 默认允许的目录：workspace, ~/.openclaw, ~/.claw, /tmp
func DefaultPolicy(workspace string) PathPolicy {
	home, _ := os.UserHomeDir()

	prefixes := []string{}
	if workspace != "" {
		prefixes = append(prefixes, workspace)
	}
	if home != "" {
		prefixes = append(prefixes,
			filepath.Join(home, ".openclaw"),
			filepath.Join(home, ".claw"),
		)
	}
	prefixes = append(prefixes, "/tmp")

	return PathPolicy{
		AllowedPrefixes: prefixes,
		DeniedPrefixes:  nil,
	}
}

// Check 检查给定路径是否符合策略
// 返回 allowed=true 表示路径在白名单内，reason 描述原因
func (p PathPolicy) Check(path string) (allowed bool, reason string) {
	if path == "" {
		return false, "路径为空"
	}

	// 解析为绝对路径
	absPath, err := filepath.Abs(path)
	if err != nil {
		return false, "无法解析路径"
	}

	// 确保路径没有 .. 逃逸（Clean 后比较）
	absPath = filepath.Clean(absPath)

	// 检查拒绝列表（优先级最高）
	for _, prefix := range p.DeniedPrefixes {
		cleanPrefix := filepath.Clean(prefix)
		if strings.HasPrefix(absPath, cleanPrefix) {
			return false, "路径在拒绝列表中"
		}
	}

	// 检查允许列表
	for _, prefix := range p.AllowedPrefixes {
		cleanPrefix := filepath.Clean(prefix)
		if strings.HasPrefix(absPath, cleanPrefix) {
			return true, "路径在白名单内"
		}
	}

	return false, "路径不在白名单内"
}
