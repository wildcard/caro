# cmdai (caro)

[![Crates.io](https://img.shields.io/crates/v/cmdai.svg)](https://crates.io/crates/cmdai)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
[![CI](https://github.com/wildcard/cmdai/workflows/CI/badge.svg)](https://github.com/wildcard/cmdai/actions)

[English](README.md) | 简体中文

> ✨ **活跃开发中** - 已发布到 crates.io，核心功能可用。访问 [caro.sh](https://caro.sh) 了解更多信息。

**cmdai**（又名 **caro**）是一个使用本地大语言模型将自然语言描述转换为安全 POSIX shell 命令的工具。使用 Rust 构建，具有极速性能、单一二进制分发、安全优先的设计理念，并针对 Apple Silicon 进行了智能平台检测优化。

```bash
$ cmdai "在 Downloads 文件夹中列出所有大于 10MB 的 PDF 文件"
# 或使用别名
$ caro "在 Downloads 文件夹中列出所有大于 10MB 的 PDF 文件"

生成的命令:
  find ~/Downloads -name "*.pdf" -size +10M -ls

是否执行此命令? (y/N) y
```

## 📋 项目状态

**当前版本:** 0.1.0（已发布到 [crates.io](https://crates.io/crates/cmdai)）

本项目处于**活跃开发**阶段，核心功能已实现并可正常使用。CLI 具备嵌入式本地推理和先进的平台感知命令生成能力。

### ✅ 已完成并发布
- ✨ **已发布到 crates.io** - 通过 `cargo install cmdai` 安装
- 🎯 核心 CLI 结构，配合全面的参数解析
- 🏗️ 模块化架构，基于 trait 的后端系统
- 🧠 **嵌入式模型后端**，支持 MLX（Apple Silicon）和 CPU 模式
- 🤖 **智能上下文循环** - 结合平台检测的迭代优化
- 🌍 **平台感知生成** - 检测操作系统、架构、可用命令
- 📍 **执行上下文检测** - 当前工作目录、Shell 类型、系统约束
- 🌐 远程后端支持（Ollama、vLLM），自动回退机制
- 🛡️ 安全验证，52 个预编译危险命令模式
- ⚙️ 配置管理，支持 TOML 格式
- 💬 交互式用户确认流程，彩色风险等级显示
- 🎬 **命令执行引擎** - 安全执行，Shell 检测
- 📄 多种输出格式（JSON、YAML、纯文本）
- 🧪 基于契约的测试结构，TDD 方法论
- 🔄 多平台 CI/CD 流水线，自动发布
- 📦 安装脚本，自动设置 `caro` 别名
- 🖥️ 跨平台检测和验证（macOS、Linux、Windows）
- 🌐 **官方网站** [caro.sh](https://caro.sh)
- 🎥 **专业演示** asciinema 录制

### 🚧 进行中
- 模型下载和缓存优化
- 命令历史和用户反馈学习
- 性能分析和优化
- 扩展安全模式库

### 📅 计划中
- 多步骤目标完成，依赖解析
- Shell 脚本生成，支持复杂工作流
- 交互式命令优化与解释
- 插件系统，支持自定义后端和验证器

## ✨ 特性

- 🚀 **快速启动** - 单一二进制，快速初始化
- 🧠 **本地 LLM 推理** - 针对 Apple Silicon（MLX）和 CPU 优化的嵌入式模型
- 🤖 **智能优化** - 2 次迭代智能循环，生成平台特定命令
- 🌍 **平台感知** - 自动检测操作系统、架构、Shell 和可用命令
- 🛡️ **安全优先** - 全面验证，52+ 危险命令模式
- 📦 **自包含** - 单一二进制分发，嵌入式模型
- 🎯 **多后端** - 可扩展系统，支持 MLX、CPU、vLLM 和 Ollama
- 💾 **模型管理** - 内置模型加载和优化
- 🌐 **跨平台** - 完整支持 macOS（包括 Apple Silicon）、Linux 和 Windows
- 🎬 **安全执行** - 可选命令执行，Shell 感知处理

## 🚀 快速开始

### 安装

#### 选项 1: 一键安装（推荐）
```bash
bash <(curl --proto '=https' --tlsv1.2 -sSfL https://setup.caro.sh)
```

或使用 wget:
```bash
bash <(wget -qO- https://setup.caro.sh)
```

这将会:
- 安装 Rust（如果尚未安装）
- 通过 cargo 安装 cmdai，启用 MLX 优化（Apple Silicon）
- 自动设置 `caro` 别名
- 配置你的 Shell（bash、zsh 或 fish）

#### 选项 2: 使用 Cargo
```bash
cargo install cmdai

# 手动添加别名到 Shell 配置文件（~/.bashrc、~/.zshrc 等）
alias caro='cmdai'
```

#### 选项 3: 预编译二进制文件
从 [GitHub Releases](https://github.com/wildcard/cmdai/releases/latest) 下载适合你平台的最新版本：
- Linux（x64、ARM64）
- macOS（Intel、Apple Silicon）
- Windows（x64）

### 从源码构建

#### 前置条件
- **Rust 1.75+** 和 Cargo
- **CMake**（用于模型推理后端）
- **macOS Apple Silicon**（可选，用于 GPU 加速）
- **Xcode**（可选，用于 Apple Silicon 上的完整 MLX GPU 支持）

### 平台特定安装

#### macOS（推荐 Apple Silicon）

完整的 macOS 安装说明（包括 GPU 加速），请参阅 [macOS 安装指南](docs/MACOS_SETUP.md)。

**快速安装:**
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 通过 Homebrew 安装 CMake
brew install cmake

# 克隆并构建
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release

# 运行
./target/release/cmdai "列出所有文件"
```

**GPU 加速（仅限 Apple Silicon）:**
- 从 App Store 安装 Xcode（Metal 编译器需要）
- 使用以下命令构建: `cargo build --release --features embedded-mlx`
- 详见 [macOS 安装指南](docs/MACOS_SETUP.md)

**注意:** 默认构建使用 stub 实现，无需 Xcode 即可立即使用。生产环境的 GPU 加速需要 Xcode。

#### Linux

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 安装依赖（Ubuntu/Debian）
sudo apt-get update
sudo apt-get install cmake build-essential

# 克隆并构建
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release
```

#### Windows

```bash
# 从 https://rustup.rs 安装 Rust
# 从 https://cmake.org/download/ 安装 CMake

# 克隆并构建
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# 构建项目（默认使用 CPU 后端）
cargo build --release

# 运行 CLI
./target/release/cmdai --version
```

### 开发命令

```bash
# 运行测试
make test

# 格式化代码
make fmt

# 运行 linter
make lint

# 构建优化版本
make build-release

# 启用调试日志运行
RUST_LOG=debug cargo run -- "你的命令"
```

## 📖 使用方法

### 基本语法
```bash
cmdai [选项] <提示语>
```

### 示例
```bash
# 基本命令生成
cmdai "列出当前目录下的所有文件"

# 指定 Shell
cmdai --shell zsh "查找大文件"

# JSON 输出（适合脚本使用）
cmdai --output json "显示磁盘使用情况"

# 调整安全级别
cmdai --safety permissive "清理临时文件"

# 自动确认危险命令
cmdai --confirm "删除旧日志文件"

# 详细模式，显示计时信息
cmdai --verbose "搜索 Python 文件"
```

### CLI 选项

| 选项 | 描述 | 状态 |
|--------|-------------|--------|
| `-s, --shell <SHELL>` | 目标 Shell（bash、zsh、fish、sh、powershell、cmd） | ✅ 已实现 |
| `--safety <LEVEL>` | 安全级别（strict、moderate、permissive） | ✅ 已实现 |
| `-o, --output <FORMAT>` | 输出格式（json、yaml、plain） | ✅ 已实现 |
| `-y, --confirm` | 自动确认危险命令 | ✅ 已实现 |
| `-v, --verbose` | 启用详细输出，显示计时信息 | ✅ 已实现 |
| `-c, --config <FILE>` | 自定义配置文件 | ✅ 已实现 |
| `--show-config` | 显示当前配置 | ✅ 已实现 |
| `--auto` | 无需确认执行 | 📅 计划中 |
| `--allow-dangerous` | 允许潜在危险命令 | 📅 计划中 |
| `--verbose` | 启用详细日志 | ✅ 可用 |

### 功能示例

```bash
# 简单命令生成
cmdai "压缩当前目录下的所有图片"

# 使用特定后端
cmdai --backend mlx "查找大型日志文件"

# 调试模式
cmdai --verbose "显示磁盘使用情况"
```

## 🏗️ 架构

### 模块结构

```
cmdai/
├── src/
│   ├── main.rs              # CLI 入口点
│   ├── backends/            # LLM 后端实现
│   │   ├── mod.rs          # 后端 trait 定义
│   │   ├── mlx.rs          # Apple Silicon MLX 后端
│   │   ├── vllm.rs         # vLLM 远程后端
│   │   └── ollama.rs       # Ollama 本地后端
│   ├── safety/             # 命令验证
│   │   └── mod.rs          # 安全验证器
│   ├── cache/              # 模型缓存
│   ├── config/             # 配置管理
│   ├── cli/                # CLI 接口
│   ├── models/             # 数据模型
│   └── execution/          # 命令执行
├── tests/                   # 基于契约的测试
└── specs/                  # 项目规范
```

### 核心组件

1. **CommandGenerator Trait** - 所有 LLM 后端的统一接口
2. **SafetyValidator** - 命令验证和风险评估
3. **后端系统** - 可扩展架构，支持多种推理引擎
4. **AgentLoop** - 结合平台检测的迭代优化
5. **ExecutionContext** - 全面的系统环境检测
6. **Model Loader** - 高效的模型初始化和管理

### 智能命令生成

cmdai 使用先进的 **2 次迭代智能循环** 生成平台适配的命令：

**第 1 次迭代：上下文感知生成**
- 检测你的操作系统（macOS、Linux、Windows）、架构和 Shell
- 识别系统上可用的命令
- 应用平台特定规则（BSD vs GNU 差异）
- 生成初始命令，带置信度评分

**第 2 次迭代：智能优化**（需要时触发）
- 从管道和链式命令中提取命令
- 获取命令特定的帮助和版本信息
- 检测并修复平台兼容性问题
- 优化复杂命令（sed、awk、xargs）

**示例流程:**
```
用户: "显示 CPU 占用前 5 的进程"
  ↓
上下文检测: macOS 14.2, arm64, zsh
  ↓
第 1 次迭代: 使用 macOS 规则生成
  ↓
智能优化: 修复 BSD sort 语法
  ↓
结果: ps aux | sort -nrk 3,3 | head -6
```

### 后端架构

```rust
#[async_trait]
trait CommandGenerator {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand, GeneratorError>;
    async fn is_available(&self) -> bool;
    fn backend_info(&self) -> BackendInfo;
}
```

## 🔧 开发

### 前置条件
- Rust 1.75+
- Cargo
- Make（可选，方便命令使用）
- Docker（可选，用于开发容器）

### 设置开发环境

```bash
# 克隆并进入项目
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# 安装依赖并构建
cargo build

# 运行测试
cargo test

# 检查格式
cargo fmt -- --check

# 运行 clippy linter
cargo clippy -- -D warnings
```

### 后端配置

cmdai 支持多个推理后端，具有自动回退机制：

#### 嵌入式后端（默认）
- **MLX**: 针对 Apple Silicon Mac（M1/M2/M3）优化
- **CPU**: 使用 Candle 框架的跨平台回退
- 模型: Qwen2.5-Coder-1.5B-Instruct（量化版）
- 无需外部依赖

#### 远程后端（可选）
在 `~/.config/cmdai/config.toml` 中配置:

```toml
[backend]
primary = "embedded"  # 或 "ollama"、"vllm"
enable_fallback = true

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"

[backend.vllm]
base_url = "http://localhost:8000"
model_name = "codellama/CodeLlama-7b-hf"
api_key = "可选的 API 密钥"
```

### 项目配置

项目使用以下配置文件：
- `Cargo.toml` - Rust 依赖和构建配置
- `~/.config/cmdai/config.toml` - 用户配置
- `clippy.toml` - Linter 规则
- `rustfmt.toml` - 代码格式化规则
- `deny.toml` - 依赖审计配置

### 测试策略

项目使用基于契约的测试：
- 单元测试，针对单个组件
- 集成测试，针对后端实现
- 契约测试，确保 trait 兼容性
- 属性测试，用于安全验证

## 🛡️ 安全特性

cmdai 包含全面的安全验证，防止危险操作：

### 已实现的安全检查
- ✅ 系统破坏模式（`rm -rf /`、`rm -rf ~`）
- ✅ Fork 炸弹检测（`:(){:|:&};:`）
- ✅ 磁盘操作（`mkfs`、`dd if=/dev/zero`）
- ✅ 权限提升检测（`sudo su`、`chmod 777 /`）
- ✅ 关键路径保护（`/bin`、`/usr`、`/etc`）
- ✅ 命令验证和清理

### 风险级别
- **安全**（绿色）- 正常操作，无需确认
- **中等**（黄色）- 严格模式下需要用户确认
- **高**（橙色）- 中等模式下需要确认
- **危急**（红色）- 严格模式下阻止，需要明确确认

### 安全配置
在 `~/.config/cmdai/config.toml` 中配置安全级别:
```toml
[safety]
enabled = true
level = "moderate"  # strict、moderate 或 permissive
require_confirmation = true
custom_patterns = ["额外", "危险", "模式"]
```

## 🤝 参与贡献

欢迎贡献代码！这是一个早期项目，有很多贡献机会。

### 贡献方向
- 🔌 后端实现
- 🛡️ 安全模式定义
- 🧪 测试覆盖扩展
- 📚 文档改进
- 🐛 Bug 修复和优化

### 如何开始
1. Fork 仓库
2. 创建功能分支
3. 进行修改并编写测试
4. 确保所有测试通过
5. 提交 Pull Request

### 开发指南
- 遵循 Rust 最佳实践
- 为新功能添加测试
- 根据需要更新文档
- 使用约定式提交信息
- 提交前运行 `make check`

## 📜 许可证

本项目采用 **GNU Affero 通用公共许可证 v3.0（AGPL-3.0）** - 详见 [LICENSE](LICENSE) 文件。

### 许可证摘要
- ✅ 商业使用
- ✅ 修改
- ✅ 分发
- ✅ 私人使用
- ⚠️ 网络使用需要披露源代码
- ⚠️ 相同许可证要求
- ⚠️ 需要说明修改内容

## 🙏 致谢

- [MLX](https://github.com/ml-explore/mlx) - Apple 的机器学习框架
- [vLLM](https://github.com/vllm-project/vllm) - 高性能 LLM 服务
- [Ollama](https://ollama.ai) - 本地 LLM 运行时
- [Hugging Face](https://huggingface.co) - 模型托管和缓存
- [clap](https://github.com/clap-rs/clap) - 命令行参数解析

## 📞 支持与社区

- 🐛 **Bug 报告**: [GitHub Issues](https://github.com/wildcard/cmdai/issues)
- 💡 **功能请求**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- 📖 **文档**: 查看 `/specs` 目录获取详细规范

## 🗺️ 路线图

### 第 1 阶段：核心结构 ✅ 完成
- [x] CLI 参数解析
- [x] 模块架构
- [x] 后端 trait 系统
- [x] 嵌入式模型命令生成

### 第 2 阶段：安全与验证 ✅ 完成
- [x] 危险模式检测（52+ 模式）
- [x] POSIX 兼容性检查
- [x] 用户确认工作流
- [x] 风险评估系统，彩色编码

### 第 3 阶段：后端集成 ✅ 完成
- [x] 嵌入式 MLX 后端（Apple Silicon）
- [x] 嵌入式 CPU 后端（跨平台）
- [x] vLLM HTTP API 支持
- [x] Ollama 本地后端
- [x] 响应解析，回退策略
- [x] 全面错误处理

### 第 4 阶段：平台智能 ✅ 完成
- [x] 执行上下文检测
- [x] 平台特定命令规则
- [x] 智能优化循环
- [x] 命令信息增强
- [x] Shell 感知执行

### 第 5 阶段：生产就绪 🚧 进行中
- [x] 发布到 crates.io
- [x] 安装脚本，别名设置
- [x] 多平台 CI/CD
- [x] 网站和文档
- [x] 专业演示
- [ ] 扩展测试覆盖
- [ ] 性能基准测试套件
- [ ] 二进制分发优化

---

**使用 Rust 构建** | **安全优先** | **开源**

> **注意**: 这是一个活跃开发的项目。特性和 API 可能会有变化。查看 [specs](specs/) 目录获取详细设计文档。
