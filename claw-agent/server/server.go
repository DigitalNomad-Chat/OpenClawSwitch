package server

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"net/http"
	"sync"
	"time"

	"claw-agent/agent"
	"claw-agent/agent/runner"
	"claw-agent/security"

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // 仅本地使用，允许所有来源
	},
}

// sessionMu prevents concurrent requests for the same session
var sessionMu sync.Map

// Server WebSocket 服务器
type Server struct {
	pool      *agent.Pool
	port      int
	httpSrv   *http.Server
	srvCtx    context.Context
	srvCancel context.CancelFunc
	authToken string               // WebSocket 认证 token，为空时跳过验证（向后兼容）
	broker    *security.ApprovalBroker // 工具执行审批 broker
	mu        sync.Mutex
	running   bool
}

// New 创建新的服务器实例
// authToken: 非空时启用 WebSocket 连接认证，客户端需通过 ?token=xxx 携带
// broker: 非空时启用工具执行审批机制
func New(pool *agent.Pool, port int, ctx context.Context, authToken string, broker *security.ApprovalBroker) *Server {
	srvCtx, srvCancel := context.WithCancel(ctx)

	if port == 0 {
		var err error
		port, err = getAvailablePort()
		if err != nil {
			log.Printf("Failed to find available port: %v", err)
			port = 0
		}
	}

	addr := fmt.Sprintf("127.0.0.1:%d", port)

	// Validate port availability immediately
	ln, err := net.Listen("tcp", addr)
	if err != nil {
		log.Printf("Port %d already in use, trying to find available port", port)
		port, err = getAvailablePort()
		if err != nil {
			log.Printf("Failed to find available port: %v", err)
			port = 0
		}
		addr = fmt.Sprintf("127.0.0.1:%d", port)
	} else {
		ln.Close()
	}

	mux := http.NewServeMux()

	s := &Server{
		pool:      pool,
		port:      port,
		srvCtx:    srvCtx,
		srvCancel: srvCancel,
		authToken: authToken,
		broker:    broker,
	}

	mux.HandleFunc("/ws", s.handleWebSocket)
	mux.HandleFunc("/health", s.handleHealth)

	s.httpSrv = &http.Server{
		Addr:         addr,
		Handler:      mux,
		ReadTimeout:  0, // infinite — LLM inference can be long
		WriteTimeout: 0, // infinite
		BaseContext: func(_ net.Listener) context.Context {
			return srvCtx
		},
	}

	return s
}

// Start 启动服务器
func (s *Server) Start() error {
	s.mu.Lock()
	if s.running {
		s.mu.Unlock()
		return fmt.Errorf("server already running")
	}
	s.running = true
	s.mu.Unlock()

	go func() {
		log.Printf("WebSocket server started on %s", s.httpSrv.Addr)
		if err := s.httpSrv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("Server error: %v", err)
		}
	}()

	return nil
}

// Port 返回服务器端口
func (s *Server) Port() int {
	return s.port
}

// Shutdown 优雅关闭服务器
func (s *Server) Shutdown(timeout time.Duration) error {
	s.mu.Lock()
	if !s.running {
		s.mu.Unlock()
		return nil
	}
	s.running = false
	s.mu.Unlock()

	s.srvCancel()

	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	return s.httpSrv.Shutdown(ctx)
}

// handleWebSocket 处理 WebSocket 连接
func (s *Server) handleWebSocket(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket upgrade error: %v", err)
		return
	}
	defer conn.Close()
	defer func() {
		// 连接断开时清理所有 pending 审批请求
		if s.broker != nil {
			s.broker.CancelAll()
		}
	}()

	log.Printf("New WebSocket connection from %s", r.RemoteAddr)

	// Token 认证验证
	if s.authToken != "" {
		token := r.URL.Query().Get("token")
		if token != s.authToken {
			log.Printf("WebSocket authentication failed from %s", r.RemoteAddr)
			conn.WriteMessage(websocket.CloseMessage,
				websocket.FormatCloseMessage(websocket.ClosePolicyViolation, "authentication failed"))
			conn.Close()
			return
		}
	}

	ctx := s.srvCtx

	for {
		conn.SetReadDeadline(time.Now().Add(5 * time.Minute))
		var req Request
		if err := conn.ReadJSON(&req); err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("WebSocket read error: %v", err)
			}
			break
		}

		log.Printf("Received request: type=%s, sessionId=%s", req.Type, req.SessionID)

		// 处理请求
		s.handleRequest(ctx, conn, req)
	}
}

// handleRequest 处理请求
func (s *Server) handleRequest(ctx context.Context, conn *websocket.Conn, req Request) {
	switch req.Type {
	case "chat":
		s.handleChat(ctx, conn, req)
	case "status":
		s.handleStatus(conn, req)
	case "tool_approval_response":
		s.handleApprovalResponse(conn, req)
	default:
		s.sendError(conn, req.ID, fmt.Sprintf("unknown request type: %s", req.Type))
	}
}

// handleChat 处理聊天请求
func (s *Server) handleChat(ctx context.Context, conn *websocket.Conn, req Request) {
	if req.Content == "" {
		s.sendError(conn, req.ID, "content is required")
		return
	}

	// Acquire session lock to prevent concurrent requests for the same session
	lockVal, _ := sessionMu.LoadOrStore(req.SessionID, &sync.Mutex{})
	mu := lockVal.(*sync.Mutex)
	mu.Lock()
	defer mu.Unlock()

	// 发送 delta 事件
	go func() {
		for evt := range s.pool.Chat(ctx, req.SessionID, req.Content) {
			if evt.Err != nil {
				s.sendError(conn, req.ID, evt.Err.Error())
				s.sendDone(conn, req.SessionID)
				return
			}

			if evt.Text != "" {
				s.sendDelta(conn, req.SessionID, evt.Text)
			}

			if evt.ToolUse != nil {
				s.sendTool(conn, req.SessionID, evt.ToolUse.Tool, evt.ToolUse.Status, evt.ToolUse.Detail)
			}

			// 发送审批请求到前端
			if evt.ApprovalRequest != nil {
				s.sendApprovalRequest(conn, req.SessionID, evt.ApprovalRequest)
			}
		}

		s.sendDone(conn, req.SessionID)
	}()
}

// handleStatus 处理状态请求
func (s *Server) handleStatus(conn *websocket.Conn, req Request) {
	sessions, err := s.pool.ListSessions(false)
	if err != nil {
		s.sendError(conn, req.ID, err.Error())
		return
	}

	s.sendResponse(conn, Response{
		ID:        req.ID,
		Type:      "status",
		SessionID: req.SessionID,
		Data: map[string]interface{}{
			"sessions": sessions,
		},
		Timestamp: time.Now().UnixMilli(),
	})
}

// 发送方法
func (s *Server) sendDelta(conn *websocket.Conn, sessionID, text string) {
	s.sendResponse(conn, Response{
		Type:      "delta",
		SessionID: sessionID,
		Data: map[string]interface{}{
			"text": text,
		},
		Timestamp: time.Now().UnixMilli(),
	})
}

func (s *Server) sendDone(conn *websocket.Conn, sessionID string) {
	s.sendResponse(conn, Response{
		Type:      "done",
		SessionID: sessionID,
		Timestamp: time.Now().UnixMilli(),
	})
}

func (s *Server) sendTool(conn *websocket.Conn, sessionID, tool, status string, detail ...string) {
	data := map[string]interface{}{
		"tool":   tool,
		"status": status,
	}
	if len(detail) > 0 && detail[0] != "" {
		data["detail"] = detail[0]
	}
	s.sendResponse(conn, Response{
		Type:      "tool",
		SessionID: sessionID,
		Data:      data,
		Timestamp: time.Now().UnixMilli(),
	})
}

func (s *Server) sendError(conn *websocket.Conn, reqID, errMsg string) {
	s.sendResponse(conn, Response{
		ID:      reqID,
		Type:    "error",
		Data:    map[string]interface{}{"error": errMsg},
		Timestamp: time.Now().UnixMilli(),
	})
}

func (s *Server) sendResponse(conn *websocket.Conn, resp Response) {
	if err := conn.WriteJSON(resp); err != nil {
		log.Printf("WebSocket write error: %v", err)
	}
}

// sendApprovalRequest 发送工具审批请求到前端
func (s *Server) sendApprovalRequest(conn *websocket.Conn, sessionID string, req *runner.ApprovalRequestEvent) {
	s.sendResponse(conn, Response{
		Type:      "tool_approval_request",
		SessionID: sessionID,
		Data: map[string]interface{}{
			"approvalId": req.ApprovalID,
			"tool":       req.Tool,
			"input":      req.Input,
			"risk":       req.Risk,
		},
		Timestamp: time.Now().UnixMilli(),
	})
}

// handleApprovalResponse 处理前端的审批响应
func (s *Server) handleApprovalResponse(conn *websocket.Conn, req Request) {
	if s.broker == nil {
		s.sendError(conn, req.ID, "审批功能未启用")
		return
	}

	approvalID, _ := req.Data["approvalId"].(string)
	if approvalID == "" {
		s.sendError(conn, req.ID, "缺少 approvalId")
		return
	}

	approved, _ := req.Data["approved"].(bool)

	decision := security.ApprovalDecision{
		Approved: approved,
		Reason:   "用户操作",
	}

	if err := s.broker.Resolve(approvalID, decision); err != nil {
		log.Printf("Failed to resolve approval %s: %v", approvalID, err)
		s.sendError(conn, req.ID, err.Error())
		return
	}

	log.Printf("Approval resolved: id=%s, approved=%v", approvalID, approved)
}

// handleHealth 处理健康检查
func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"port":   s.port,
	})
}

// getAvailablePort 获取可用端口
func getAvailablePort() (int, error) {
	// 从 34567 开始尝试可用端口
	for port := 34567; port < 35000; port++ {
		ln, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", port))
		if err == nil {
			ln.Close()
			return port, nil
		}
	}
	return 0, fmt.Errorf("no available port found in range 34567-34999")
}

// Request 请求消息
type Request struct {
	ID        string                 `json:"id"`
	Type      string                 `json:"type"`
	SessionID string                 `json:"sessionId"`
	Content   string                 `json:"content,omitempty"`
	Data      map[string]interface{} `json:"data,omitempty"`
	Timestamp int64                  `json:"timestamp"`
}

// Response 响应消息
type Response struct {
	ID        string                 `json:"id,omitempty"`
	Type      string                 `json:"type"`
	SessionID string                 `json:"sessionId"`
	Data      map[string]interface{} `json:"data,omitempty"`
	Timestamp int64                  `json:"timestamp"`
}
