# 商业化分发指南 / Commercial Distribution Guide

## 📋 已创建的文件清单

以下文件已为你的商业化分发准备完毕：

### 1. 核心声明文件

| 文件 | 位置 | 说明 |
|------|------|------|
| `LICENSE.COMBINED` | 项目根目录 | 完整的 MIT License + 你的版权声明 |
| `ATTRIBUTION.md` | 项目根目录 | 详细的版权归属声明 |
| `THIRD-PARTY-NOTICES.md` | 项目根目录 | 第三方组件完整许可声明 |
| `README.COMMERCIAL.md` | 项目根目录 | 商业版 README 模板 |

### 2. 打包资源文件

| 文件 | 位置 | 说明 |
|------|------|------|
| `LICENSE` | `src-tauri/resources/` | 将被打包进应用 |
| `ATTRIBUTION.md` | `src-tauri/resources/` | 将被打包进应用 |
| `THIRD-PARTY-NOTICES.md` | `src-tauri/resources/` | 将被打包进应用 |

### 3. 自动化脚本

| 文件 | 位置 | 说明 |
|------|------|------|
| `prepare-licenses.sh` | `scripts/` | macOS/Linux 自动准备脚本 |
| `prepare-licenses.bat` | `scripts/` | Windows 自动准备脚本 |

---

## ⚠️ 使用前必做：替换占位符

在分发前，请务必替换以下占位符：

### 需要替换的内容

| 占位符 | 替换为 | 出现位置 |
|--------|--------|----------|
| `[Your Name / Your Company Name]` | 你的名字或公司名称 | LICENSE, ATTRIBUTION.md |
| `[你的名字/公司]` | 你的名字或公司名称 | README.COMMERCIAL.md |
| `[你的品牌名]` | 你的产品品牌名 | README.COMMERCIAL.md, tauri.conf.json |
| `[Your Repository URL]` | 你的 GitHub 仓库地址 | ATTRIBUTION.md |
| `YourOrg/YourRepo` | 你的组织名/仓库名 | README.COMMERCIAL.md |
| `[your-email@example.com]` | 你的联系邮箱 | README.COMMERCIAL.md |

### 快速替换命令

```bash
# 在项目根目录执行
cd ~/办公/github/OpenClawSwitch

# 替换占位符（示例，请根据实际情况修改）
find . -type f \( -name "*.md" -o -name "LICENSE*" \) -exec sed -i '' \
  's/\[Your Name\/Your Company Name\]/你的名字或公司/g' {} \;

find . -type f \( -name "*.md" -o -name "LICENSE*" \) -exec sed -i '' \
  's/\[Your Brand Name\]/你的品牌名/g' {} \;
```

---

## 🚀 分发前检查清单

### Step 1: 更新品牌信息

```
□ 更新 package.json 中的 name 和 description
□ 更新 src-tauri/tauri.conf.json 中的 productName
□ 更新 src-tauri/Cargo.toml 中的 name 和 description
□ 更新 Bundle ID (com.yourbrand.app)
□ 替换所有图标和 Logo
□ 更新 index.html 中的标题
```

### Step 2: 许可文件准备

```
□ 运行 ./scripts/prepare-licenses.sh (macOS/Linux)
   或 scripts\prepare-licenses.bat (Windows)
□ 检查 src-tauri/resources/ 目录包含：
   - LICENSE
   - ATTRIBUTION.md
   - THIRD-PARTY-NOTICES.md
□ 更新 LICENSE.COMBINED 中的占位符
□ 确认 ATTRIBUTION.md 中的信息正确
```

### Step 3: README 更新

```
□ 使用 README.COMMERCIAL.md 作为模板
□ 替换所有占位符
□ 更新截图（如有品牌改动）
□ 添加你的联系方式
□ 添加官方网站链接（如有）
```

### Step 4: 构建测试

```bash
# macOS/Linux
npm run tauri:build

# Windows
npm run tauri:build

# 验证输出
# 检查 dist/ 或 src-tauri/target/release/bundle/ 目录
```

### Step 5: 法律合规检查

```
□ LICENSE 文件已包含在安装包中
□ 应用内"关于"页面可查看许可信息
□ README 有"基于原项目"声明
□ 没有使用原项目商标/名称
□ 所有依赖协议已检查（无 GPL/AGPL）
```

---

## 📦 构建和分发

### 构建命令

```bash
# 开发构建
npm run tauri:dev

# 生产构建
npm run tauri:build

# macOS 通用二进制（如已配置）
npm run tauri:build:mac

# Windows 构建
npm run tauri:build:windows
```

### 分发位置

构建完成后，安装包位于：

| 平台 | 位置 |
|------|------|
| macOS | `src-tauri/target/release/bundle/dmg/` |
| macOS | `src-tauri/target/release/bundle/macos/` |
| Windows | `src-tauri/target/release/bundle/msi/` |
| Windows | `src-tauri/target/release/bundle/nsis/` |

---

## 🎯 商业化建议

### 定价策略

| 模式 | 适用场景 | 建议定价 |
|------|----------|----------|
| 一次性买断 | 工具类应用 | ¥29-199 |
| 订阅制（月付） | 持续更新服务 | ¥9-29/月 |
| 订阅制（年付） | 持续更新服务 | ¥99-299/年 |
| 免费+增值 | 扩大用户基础 | 基础免费，高级收费 |

### 分发渠道

```
□ 自建官网下载
□ GitHub Releases
□ 微信群/QQ群分发
□ 微信公众号
□ 知识星球/社群
```

### 用户协议

建议在应用内添加：

```
1. 服务条款 (Terms of Service)
2. 隐私政策 (Privacy Policy)
3. 退款政策 (Refund Policy)
4. 免责声明 (Disclaimer)
```

---

## 📄 文件结构总览

```
OpenClawSwitch/
├── LICENSE                    ← 原 MIT License (保留)
├── LICENSE.COMBINED           ← 完整版权声明 (新建)
├── ATTRIBUTION.md             ← 版权归属声明 (新建)
├── THIRD-PARTY-NOTICES.md     ← 第三方许可 (新建)
├── README.md                  ← 原 README (保留)
├── README.COMMERCIAL.md       ← 商业版模板 (新建)
├── COMMERCIAL-DISTRIBUTION-GUIDE.md ← 本文档 (新建)
├── package.json               ← 需更新
├── src-tauri/
│   ├── tauri.conf.json        ← 需更新
│   ├── Cargo.toml             ← 需更新
│   └── resources/
│       ├── LICENSE            ← 已创建
│       ├── ATTRIBUTION.md     ← 已创建
│       └── THIRD-PARTY-NOTICES.md ← 已创建
└── scripts/
    ├── prepare-licenses.sh    ← 已创建 (可执行)
    └── prepare-licenses.bat   ← 已创建
```

---

## ✅ MIT License 合规总结

### 你必须做的

1. ✅ 保留原 MIT License 文件
2. ✅ 在分发版本中包含 LICENSE 文件
3. ✅ 不删除原作者版权声明

### 你可以做的

1. ✅ 收费（一次性/订阅）
2. ✅ 闭源分发
3. ✅ 更换品牌
4. ✅ 修改功能
5. ✅ 宣传销售

### 你不应该做的

1. ❌ 声称完全原创
2. ❌ 删除 MIT License 文件
3. ❌ 使用原项目商标
4. ❌ 阻止用户查看 LICENSE

---

## 📞 后续支持

如有疑问，请检查：

1. MIT License 官方文本：https://opensource.org/licenses/MIT
2. Tauri 文档：https://tauri.app/
3. 原项目：https://github.com/RongleCat/OpenClawSwitch

---

**创建日期**: 2025-03-21
**适用项目**: OpenClawSwitch (基于 RongleCat/OpenClawSwitch)
**许可协议**: MIT License
