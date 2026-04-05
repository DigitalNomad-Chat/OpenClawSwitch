package engine

import (
	"context"
	"errors"

	aitypes "claw-agent/ai/types"
)

// AskApprovalFunc 审批回调函数签名
// 返回 true 表示允许执行，false 表示跳过
type AskApprovalFunc func(ctx context.Context, call aitypes.ToolCall) (bool, error)

// ToolCallbacks emits progress events around tool execution.
type ToolCallbacks struct {
	OnStart      func(call aitypes.ToolCall)
	OnFinish     func(result aitypes.ToolResultMessage)
	AskApproval  AskApprovalFunc // nil 时所有工具自动放行（向后兼容）
}

// ExecuteToolCalls runs each tool call in order and returns result messages.
// 如果 AskApproval 回调存在且返回 false，跳过该工具执行。
func ExecuteToolCalls(ctx context.Context, calls []aitypes.ToolCall, tools ToolSet, cb ToolCallbacks) ([]aitypes.ToolResultMessage, error) {
	results := make([]aitypes.ToolResultMessage, 0, len(calls))

	for _, call := range calls {
		if cb.OnStart != nil {
			cb.OnStart(call)
		}

		toolFn, ok := tools[call.Name]
		if !ok {
			result := aitypes.ToolResultMessage{
				ToolCallID: call.ID,
				ToolName:   call.Name,
				IsError:    true,
				Content:    []aitypes.ContentBlock{aitypes.TextContent{Text: "tool not found"}},
			}
			results = append(results, result)
			if cb.OnFinish != nil {
				cb.OnFinish(result)
			}
			continue
		}

		// 审批检查（向后兼容：nil 回调时自动放行）
		if cb.AskApproval != nil {
			approved, err := cb.AskApproval(ctx, call)
			if err != nil {
				result := aitypes.ToolResultMessage{
					ToolCallID: call.ID,
					ToolName:   call.Name,
					IsError:    true,
					Content:    []aitypes.ContentBlock{aitypes.TextContent{Text: err.Error()}},
				}
				results = append(results, result)
				if cb.OnFinish != nil {
					cb.OnFinish(result)
				}
				continue
			}
			if !approved {
				result := aitypes.ToolResultMessage{
					ToolCallID: call.ID,
					ToolName:   call.Name,
					IsError:    true,
					Content:    []aitypes.ContentBlock{aitypes.TextContent{Text: "用户拒绝了此操作"}},
				}
				results = append(results, result)
				if cb.OnFinish != nil {
					cb.OnFinish(result)
				}
				continue
			}
		}

		content, err := toolFn(ctx, call)
		result := aitypes.ToolResultMessage{ToolCallID: call.ID, ToolName: call.Name, Content: []aitypes.ContentBlock{content}}
		if err != nil {
			result.IsError = true
			result.Content = []aitypes.ContentBlock{aitypes.TextContent{Text: err.Error()}}
		}
		results = append(results, result)
		if cb.OnFinish != nil {
			cb.OnFinish(result)
		}
	}

	if len(calls) > 0 && len(results) == 0 {
		return nil, errors.New("tool execution produced no results")
	}

	return results, nil
}
