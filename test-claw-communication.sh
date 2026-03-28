#!/bin/bash

# Claw Agent 通信测试脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAW_AGENT="$SCRIPT_DIR/claw-agent-bin"

echo "========================================="
echo "Claw Agent 通信测试"
echo "========================================="
echo ""

# 检查二进制文件
if [ ! -f "$CLAW_AGENT" ]; then
    echo "❌ 错误: Claw Agent 二进制文件不存在"
    echo "   位置: $CLAW_AGENT"
    exit 1
fi

echo "✅ 找到 Claw Agent 二进制文件"
echo "   路径: $CLAW_AGENT"
echo "   大小: $(ls -lh "$CLAW_AGENT" | awk '{print $5}')"
echo ""

# 测试版本命令
echo "========================================="
echo "测试 1: 版本命令"
echo "========================================="
if "$CLAW_AGENT" --version; then
    echo "✅ 版本命令测试通过"
else
    echo "❌ 版本命令测试失败"
    exit 1
fi
echo ""

# 测试帮助命令
echo "========================================="
echo "测试 2: 帮助命令"
echo "========================================="
if "$CLAW_AGENT" --help > /dev/null 2>&1; then
    echo "✅ 帮助命令测试通过"
else
    echo "❌ 帮助命令测试失败"
    exit 1
fi
echo ""

# 测试 Agent 模式启动
echo "========================================="
echo "测试 3: Agent 模式启动"
echo "========================================="

# 创建临时配置目录
TEMP_CONFIG_DIR=$(mktemp -d)
mkdir -p "$TEMP_CONFIG_DIR"

# 创建测试配置
cat > "$TEMP_CONFIG_DIR/config.yaml" <<EOF
provider: anthropic
model: claude-sonnet-4-6
workspace: $TEMP_CONFIG_DIR/workspace
providers:
  anthropic:
    api_key: "test-key"
EOF

echo "临时配置目录: $TEMP_CONFIG_DIR"
echo ""

# 启动 Agent（后台）
echo "启动 Claw Agent..."
"$CLAW_AGENT" --agent --port 34567 --config "$TEMP_CONFIG_DIR/config.yaml" > /tmp/claw-agent.log 2>&1 &
CLAW_PID=$!

echo "Agent PID: $CLAW_PID"
echo ""

# 等待启动
echo "等待 Agent 启动..."
sleep 3

# 检查进程是否运行
if kill -0 $CLAW_PID 2>/dev/null; then
    echo "✅ Agent 进程运行中 (PID: $CLAW_PID)"
else
    echo "❌ Agent 进程未运行"
    cat /tmp/claw-agent.log
    exit 1
fi
echo ""

# 测试健康检查
echo "========================================="
echo "测试 4: 健康检查端点"
echo "========================================="
HEALTH_URL="http://127.0.0.1:34567/health"

if command -v curl > /dev/null 2>&1; then
    echo "使用 curl 测试健康检查..."
    if curl -s "$HEALTH_URL" | grep -q "ok"; then
        echo "✅ 健康检查通过"
        curl -s "$HEALTH_URL" | jq . 2>/dev/null || curl -s "$HEALTH_URL"
    else
        echo "❌ 健康检查失败"
        echo "响应:"
        curl -s "$HEALTH_URL"
        kill $CLAW_PID 2>/dev/null
        exit 1
    fi
else
    echo "⚠️  curl 未安装，跳过健康检查"
fi
echo ""

# 测试 WebSocket 连接（如果 websocat 可用）
echo "========================================="
echo "测试 5: WebSocket 端点"
echo "========================================="
if command -v websocat > /dev/null 2>&1; then
    echo "使用 websocat 测试 WebSocket 连接..."
    if echo '{"type":"status","sessionId":"test"}' | websocat ws://127.0.0.1:34567/ws --one-line; then
        echo "✅ WebSocket 连接测试通过"
    else
        echo "❌ WebSocket 连接测试失败"
    fi
else
    echo "⚠️  websocat 未安装，跳过 WebSocket 测试"
    echo "   安装: brew install websocat"
fi
echo ""

# 清理
echo "========================================="
echo "清理"
echo "========================================="
echo "停止 Agent (PID: $CLAW_PID)..."
kill $CLAW_PID 2>/dev/null
sleep 1

if kill -0 $CLAW_PID 2>/dev/null; then
    echo "强制停止 Agent..."
    kill -9 $CLAW_PID 2>/dev/null
fi

echo "清理临时文件..."
rm -rf "$TEMP_CONFIG_DIR"
echo ""

echo "========================================="
echo "✅ 所有测试完成"
echo "========================================="
