package security

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"sync"
	"sync/atomic"
	"time"
)

// ApprovalDecision 表示审批结果
type ApprovalDecision struct {
	Approved bool
	Reason   string
}

// ApprovalRequest 表示需要用户审批的工具调用请求
type ApprovalRequest struct {
	ID      string         // 唯一标识
	Tool    string         // 工具名称
	Input   string         // 工具输入摘要
	Args    map[string]any // 原始参数
	Risk    string         // 风险等级: "low", "medium", "high"
}

// ApprovalBroker 管理工具执行审批请求的全局单例
// 通过 channel 阻塞机制实现 goroutine 间的审批同步
type ApprovalBroker struct {
	pending  map[string]chan ApprovalDecision
	counter  atomic.Int64
	mu       sync.Mutex
	log      *slog.Logger
}

// NewApprovalBroker 创建新的审批 broker
func NewApprovalBroker() *ApprovalBroker {
	return &ApprovalBroker{
		pending: make(map[string]chan ApprovalDecision),
		log:     slog.With("component", "approval_broker"),
	}
}

// Ask 注册审批请求并阻塞等待用户响应
// 支持通过 context 取消和 5 分钟超时自动拒绝
func (b *ApprovalBroker) Ask(ctx context.Context, req ApprovalRequest) (ApprovalDecision, error) {
	ch := make(chan ApprovalDecision, 1)

	b.mu.Lock()
	b.pending[req.ID] = ch
	b.mu.Unlock()

	b.log.Info("approval request pending", "id", req.ID, "tool", req.Tool, "risk", req.Risk)

	defer func() {
		b.mu.Lock()
		delete(b.pending, req.ID)
		b.mu.Unlock()
	}()

	// 5 分钟超时
	timer := time.NewTimer(5 * time.Minute)
	defer timer.Stop()

	select {
	case decision := <-ch:
		return decision, nil
	case <-timer.C:
		b.log.Warn("approval timeout", "id", req.ID, "tool", req.Tool)
		return ApprovalDecision{
			Approved: false,
			Reason:   "审批超时（5分钟），操作已自动拒绝",
		}, nil
	case <-ctx.Done():
		b.log.Info("approval cancelled by context", "id", req.ID)
		return ApprovalDecision{}, ctx.Err()
	}
}

// Resolve 由 server 收到前端响应后调用，解除 goroutine 阻塞
func (b *ApprovalBroker) Resolve(id string, decision ApprovalDecision) error {
	b.mu.Lock()
	ch, ok := b.pending[id]
	b.mu.Unlock()

	if !ok {
		return fmt.Errorf("approval %s not found or already resolved", id)
	}

	select {
	case ch <- decision:
		b.log.Info("approval resolved", "id", id, "approved", decision.Approved)
	default:
		// channel 已关闭或已满（不应发生，但做防御性处理）
	}

	return nil
}

// CancelAll 取消所有 pending 的审批请求（用于连接断开清理）
func (b *ApprovalBroker) CancelAll() {
	b.mu.Lock()
	defer b.mu.Unlock()

	timeoutDecision := ApprovalDecision{
		Approved: false,
		Reason:   "连接断开，操作已取消",
	}

	for id, ch := range b.pending {
		select {
		case ch <- timeoutDecision:
		default:
		}
		b.log.Info("approval cancelled", "id", id)
		delete(b.pending, id)
	}
}

// NextID 生成全局唯一的审批 ID
func (b *ApprovalBroker) NextID() string {
	n := b.counter.Add(1)
	return fmt.Sprintf("apr-%d", n)
}

// PendingCount 返回当前等待中的审批请求数量
func (b *ApprovalBroker) PendingCount() int {
	b.mu.Lock()
	defer b.mu.Unlock()
	return len(b.pending)
}

// ErrRequestNotFound 审批请求不存在
var ErrRequestNotFound = errors.New("approval request not found")
