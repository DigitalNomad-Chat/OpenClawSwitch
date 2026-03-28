#!/bin/bash
set -e

# 预构建脚本：复制 claw-agent 到资源目录
# 在 Tauri 构建前运行此脚本确保 claw-agent 是最新版本

echo "📦 准备 claw-agent 资源..."

CLAW_AGENT_SOURCE="claw-agent/claw-agent"
CLAW_AGENT_DEST="src-tauri/resources/claw-agent/claw-agent"
CLAW_DATA_SOURCE="resources/claw-data"
CLAW_DATA_DEST="src-tauri/resources/claw-data"

# 检查 claw-agent 源文件是否存在
if [ ! -f "$CLAW_AGENT_SOURCE" ]; then
    echo "❌ 错误: 找不到 claw-agent 源文件: $CLAW_AGENT_SOURCE"
    echo "   请先构建 claw-agent: cd claw-agent && go build -o claw-agent"
    exit 1
fi

# 复制 claw-agent
echo "   复制 claw-agent 到资源目录..."
mkdir -p "src-tauri/resources/claw-agent"
cp "$CLAW_AGENT_SOURCE" "$CLAW_AGENT_DEST"
chmod +x "$CLAW_AGENT_DEST"

# 复制 claw-data
if [ -d "$CLAW_DATA_SOURCE" ]; then
    echo "   复制 claw-data 到资源目录..."
    rm -rf "$CLAW_DATA_DEST"
    cp -r "$CLAW_DATA_SOURCE" "$CLAW_DATA_DEST"
fi

echo "✅ claw-agent 资源准备完成"
echo "   - $(basename $CLAW_AGENT_DEST) ($(stat -f%z "$CLAW_AGENT_DEST" 2>/dev/null || stat -c%s "$CLAW_AGENT_DEST" 2>/dev/null || echo 'N/A') bytes)"
