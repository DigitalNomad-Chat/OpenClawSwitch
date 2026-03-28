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

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // 仅本地使用，允许所有来源
	},
}

// Server WebSocket 服务器
type Server struct {
	pool    *agent.Pool
	port    int
	mu      sync.Mutex
	running bool
}

// New 创建新的服务器实例
func New(pool *agent.Pool, port int) *Server {
	if port == 0 {
		port = getAvailablePort()
	}
	return &Server{
		pool: pool,
		port: port,
	}
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

	mux := http.NewServeMux()
	mux.HandleFunc("/ws", s.handleWebSocket)
	mux.HandleFunc("/health", s.handleHealth)

	server := &http.Server{
		Addr:         fmt.Sprintf("127.0.0.1:%d", s.port),
		Handler:      mux,
		ReadTimeout:  60 * time.Second,
		WriteTimeout: 60 * time.Second,
	}

	go func() {
		log.Printf("WebSocket server started on %s", server.Addr)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("Server error: %v", err)
		}
	}()

	return nil
}

// Port 返回服务器端口
func (s *Server) Port() int {
	return s.port
}

// handleWebSocket 处理 WebSocket 连接
func (s *Server) handleWebSocket(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket upgrade error: %v", err)
		return
	}
	defer conn.Close()

	log.Printf("New WebSocket connection from %s", r.RemoteAddr)

	ctx := context.Background()

	for {
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

	// 发送 delta 事件
	go func() {
		for evt := range s.pool.Chat(ctx, req.SessionID, req.Content) {
			if evt.Err != nil {
				s.sendError(conn, req.ID, evt.Err.Error())
				return
			}

			if evt.Text != "" {
				s.sendDelta(conn, req.SessionID, evt.Text)
			}

			if evt.ToolUse != nil {
				s.sendTool(conn, req.SessionID, evt.ToolUse.Tool, "running")
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

func (s *Server) sendTool(conn *websocket.Conn, sessionID, tool, status string) {
	s.sendResponse(conn, Response{
		Type:      "tool",
		SessionID: sessionID,
		Data: map[string]interface{}{
			"tool":   tool,
			"status": status,
		},
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

// handleHealth 处理健康检查
func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status": "ok",
		"port":   s.port,
	})
}

// getAvailablePort 获取可用端口
func getAvailablePort() int {
	// 从 34567 开始尝试可用端口
	for port := 34567; port < 35000; port++ {
		ln, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", port))
		if err == nil {
			ln.Close()
			return port
		}
	}
	return 34567 // 默认端口
}

// Request 请求消息
type Request struct {
	ID        string `json:"id"`
	Type      string `json:"type"`
	SessionID string `json:"sessionId"`
	Content   string `json:"content,omitempty"`
	Timestamp int64  `json:"timestamp"`
}

// Response 响应消息
type Response struct {
	ID        string `json:"id,omitempty"`
	Type      string `json:"type"`
	SessionID string `json:"sessionId"`
	Data      map[string]interface{} `json:"data,omitempty"`
	Timestamp int64  `json:"timestamp"`
}
