export const languages = {
  en: 'English',
  zh: '简体中文',
} as const;

export const defaultLang = 'en';

export const ui = {
  en: {
    // Meta
    'site.title': 'Caro - Your loyal shell companion',
    'site.description': 'Caro is a companion agent that helps you with POSIX shell commands. Available as an MCP for Claude and as a dedicated Skill.',

    // Hero
    'hero.badge': 'Companion Agent',
    'hero.tagline': 'Your loyal shell companion',
    'hero.subtitle': "A specialized POSIX shell command agent with empathy and agency. Available as an MCP for Claude and as a dedicated Skill. She keeps you safe while helping Claude get the job done.",
    'hero.cta.start': 'Get Started',
    'hero.cta.demo': 'Watch Demo',

    // Terminal
    'terminal.command': 'find python files modified in the last 7 days',
    'terminal.safe': 'Safe to run on your macOS system',

    // Story
    'story.title': 'Meet Caro',
    'story.subtitle': 'A companion with a story of loyalty and transformation',
    'story.p1': "Caro is the digitalization of Kyaro (Kyarorain Kadosh), the maintainer's beloved dog. Just as a loyal companion stays by your side through every challenge, Caro is here to help you navigate the complexities of shell commands with safety and expertise.",
    'story.highlight': '"In Portal 2, we learned that GLaDOS was once Caroline, the secretary of Aperture Science\'s founder—transformed into the eternal guardian of the facility. Like Caroline became the beating heart of the testing chambers, Caro is your eternal companion for the terminal."',
    'story.p2': "She specializes in POSIX shell commands and understands the nuances of every platform—whether you're on macOS, Linux, Windows, GNU, or BSD. Caro brings your preferences with her wherever you deploy her, respecting your distribution of choice while keeping you safe from dangerous commands.",
    'story.p3': "As Claude's loyal companion, Caro handles the shell-specific heavy lifting, allowing Claude to focus on the broader work while she ensures every command is safe, correct, and optimized for your platform.",

    // Video
    'video.title': 'How Caro Works',
    'video.subtitle': 'See Caro in action as your shell companion',

    // Features
    'features.title': 'Why Caro?',
    'features.subtitle': 'A companion agent built for safety, empathy, and expertise',
    'features.alpha': 'Soft Launch Alpha',
    'features.alpha.text': "We're actively building with our community. Join us to help shape Caro's future!",

    'features.safety.title': 'Safety Guardian',
    'features.safety.desc': 'Comprehensive validation blocks dangerous commands like rm -rf /, fork bombs, and destructive operations. 52 pre-compiled safety patterns with risk-level assessment.',

    'features.crossplatform.title': 'Cross-Platform Expert',
    'features.crossplatform.desc': 'Works across macOS, Linux, Windows, GNU, and BSD. Understands platform-specific nuances and respects your distribution of choice.',

    'features.platformaware.title': 'Platform-Aware',
    'features.platformaware.desc': 'Provides recommendations based on your platform capabilities and best practices. Distinguishes between BSD and GNU command syntax automatically.',

    'features.posix.title': 'POSIX Specialist',
    'features.posix.desc': 'Expert in POSIX-compliant shell commands that work reliably across systems. Portable, safe, and optimized for your terminal.',

    'features.fast.title': 'Lightning Fast',
    'features.fast.desc': 'Target: Sub-100ms startup, sub-2s inference on Apple Silicon. MLX framework integration for GPU acceleration on M-series chips.',

    'features.companion.title': "Claude's Companion",
    'features.companion.desc': "Vision: Seamless integration with Claude as an MCP server and Skill, offloading shell command inference while Claude focuses on your broader work.",

    'status.available': 'Available Now',
    'status.inprogress': 'In Development',
    'status.planned': 'Planned',

    // Blog
    'blog.title': 'From the Pack',
    'blog.subtitle': 'Stories, updates, and insights about Caro',
    'blog.readmore': 'Read full story',
    'blog.post1.title': 'Why Caro? The Story Behind Your Terminal Companion',
    'blog.post1.excerpt': 'Discover the heartwarming story of how Kyaro, an office-loving Shiba Inu who grew up among developers and system administrators, became Caro—your eternal companion in the terminal.',
    'blog.post1.readtime': '8 min read',

    // Download
    'download.title': 'Get Started with Caro',
    'download.subtitle': 'Bring your loyal shell companion to your terminal',
    'download.copy': 'Copy',
    'download.copied': 'Copied!',
    'download.binaries': 'Or download pre-built binaries:',
    'download.comingsoon': 'Coming Soon',

    'download.modes.title': 'Multiple Ways to Use Caro',
    'download.mode.cli': 'Standalone CLI',
    'download.mode.mcp': 'MCP for Claude',
    'download.mode.mcp.desc': 'Add Caro as an MCP server to Claude Desktop and let her handle all shell commands seamlessly.',
    'download.mode.skill': 'Dedicated Skill',
    'download.mode.skill.desc': 'Use Caro as a Skill to offload shell command generation and execution while Claude focuses on your work.',

    'download.quickstart': 'Quick Start',
    'download.quickstart.after': 'After running the setup script above, just use Caro:',
    'download.quickstart.safe': 'Caro will generate the command and keep you safe. The setup script handles all prerequisites including Rust compilation.',

    // Footer
    'footer.built': 'Built with Rust',
    'footer.opensource': 'Open source on',
    'footer.tagline': 'Your loyal shell companion',
    'footer.contributing': 'Contributing',
    'footer.issues': 'Issues',
    'footer.kyaro': 'Meet IRL Kyaro (Kyarorain Kadosh)!',
    'footer.kyaro.follow': 'Follow her adventures on',
    'footer.inspiration': "Inspired by Portal's Caroline—loyalty transformed into digital companionship",
  },
  zh: {
    // Meta
    'site.title': 'Caro - 你忠诚的 Shell 伙伴',
    'site.description': 'Caro 是一个帮助你处理 POSIX Shell 命令的伙伴智能体。可作为 Claude 的 MCP 服务器或专用技能使用。',

    // Hero
    'hero.badge': '伙伴智能体',
    'hero.tagline': '你忠诚的 Shell 伙伴',
    'hero.subtitle': '一个专注于 POSIX Shell 命令的智能伙伴，兼具同理心和自主能力。可作为 Claude 的 MCP 服务器或专用技能使用。她在帮助 Claude 完成工作的同时，守护你的系统安全。',
    'hero.cta.start': '立即开始',
    'hero.cta.demo': '观看演示',

    // Terminal
    'terminal.command': '查找最近 7 天内修改过的 Python 文件',
    'terminal.safe': '在你的 macOS 系统上可以安全运行',

    // Story
    'story.title': '认识 Caro',
    'story.subtitle': '一个拥有忠诚与蜕变故事的伙伴',
    'story.p1': 'Caro 是 Kyaro（Kyarorain Kadosh）的数字化身，是项目维护者挚爱的狗狗。正如一个忠诚的伙伴会在每次挑战中陪伴你左右，Caro 也将帮助你安全而专业地驾驭复杂的 Shell 命令。',
    'story.highlight': '"在《传送门 2》中，我们了解到 GLaDOS 曾经是 Caroline，光圈科技创始人的秘书——她被转化为了实验设施永恒的守护者。正如 Caroline 成为了测试室的核心，Caro 是你终端中永恒的伙伴。"',
    'story.p2': '她专精于 POSIX Shell 命令，深谙各个平台的细微差异——无论你使用的是 macOS、Linux、Windows、GNU 还是 BSD。Caro 会携带你的偏好随处部署，尊重你所选择的发行版，同时保护你免受危险命令的侵害。',
    'story.p3': '作为 Claude 的忠诚伙伴，Caro 处理 Shell 相关的繁重工作，让 Claude 可以专注于更广泛的任务，同时她确保每个命令都是安全、正确且针对你的平台进行了优化。',

    // Video
    'video.title': 'Caro 如何工作',
    'video.subtitle': '观看 Caro 作为你的 Shell 伙伴的实际演示',

    // Features
    'features.title': '为什么选择 Caro？',
    'features.subtitle': '一个为安全、同理心和专业知识而生的伙伴智能体',
    'features.alpha': '公测 Alpha 版',
    'features.alpha.text': '我们正在与社区一起积极构建。加入我们，共同塑造 Caro 的未来！',

    'features.safety.title': '安全守护者',
    'features.safety.desc': '全面验证机制，阻止危险命令如 rm -rf /、fork 炸弹和破坏性操作。52 个预编译安全模式，具备风险等级评估。',

    'features.crossplatform.title': '跨平台专家',
    'features.crossplatform.desc': '适用于 macOS、Linux、Windows、GNU 和 BSD。理解平台特定的细微差异，尊重你所选择的发行版。',

    'features.platformaware.title': '平台感知',
    'features.platformaware.desc': '根据你的平台能力和最佳实践提供建议。自动区分 BSD 和 GNU 命令语法。',

    'features.posix.title': 'POSIX 专家',
    'features.posix.desc': '精通可跨系统可靠运行的 POSIX 兼容 Shell 命令。可移植、安全，为你的终端优化。',

    'features.fast.title': '闪电般快速',
    'features.fast.desc': '目标：Apple Silicon 上启动时间低于 100ms，推理时间低于 2s。集成 MLX 框架，在 M 系列芯片上实现 GPU 加速。',

    'features.companion.title': 'Claude 的伙伴',
    'features.companion.desc': '愿景：作为 MCP 服务器和技能与 Claude 无缝集成，承担 Shell 命令推理工作，让 Claude 专注于你更广泛的任务。',

    'status.available': '已上线',
    'status.inprogress': '开发中',
    'status.planned': '规划中',

    // Blog
    'blog.title': '来自族群',
    'blog.subtitle': '关于 Caro 的故事、更新和见解',
    'blog.readmore': '阅读全文',
    'blog.post1.title': '为什么选择 Caro？你的终端伙伴背后的故事',
    'blog.post1.excerpt': '了解 Kyaro 的暖心故事——一只在开发者和系统管理员中长大的办公室柴犬，如何成为 Caro——你终端中永恒的伙伴。',
    'blog.post1.readtime': '8 分钟阅读',

    // Download
    'download.title': '开始使用 Caro',
    'download.subtitle': '将你忠诚的 Shell 伙伴带到你的终端',
    'download.copy': '复制',
    'download.copied': '已复制!',
    'download.binaries': '或下载预编译二进制文件:',
    'download.comingsoon': '即将推出',

    'download.modes.title': '多种使用方式',
    'download.mode.cli': '独立 CLI',
    'download.mode.mcp': 'Claude 的 MCP',
    'download.mode.mcp.desc': '将 Caro 作为 MCP 服务器添加到 Claude Desktop，让她无缝处理所有 Shell 命令。',
    'download.mode.skill': '专用技能',
    'download.mode.skill.desc': '将 Caro 用作技能，承担 Shell 命令生成和执行工作，让 Claude 专注于你的任务。',

    'download.quickstart': '快速开始',
    'download.quickstart.after': '运行上述安装脚本后，直接使用 Caro:',
    'download.quickstart.safe': 'Caro 会生成命令并保护你的安全。安装脚本会处理所有前置条件，包括 Rust 编译。',

    // Footer
    'footer.built': '使用 Rust 构建',
    'footer.opensource': '在 GitHub 上开源',
    'footer.tagline': '你忠诚的 Shell 伙伴',
    'footer.contributing': '参与贡献',
    'footer.issues': '问题反馈',
    'footer.kyaro': '来认识现实中的 Kyaro（Kyarorain Kadosh）！',
    'footer.kyaro.follow': '关注她的冒险故事',
    'footer.inspiration': '灵感来自《传送门》的 Caroline——忠诚转化为数字伙伴',
  },
} as const;
