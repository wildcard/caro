# Caro Shell Plugins

Shell plugin integrations for popular shell frameworks.

## Automatic Installation

When you install caro using the setup script:

```bash
bash <(curl -sSfL https://setup.caro.sh)
```

The installer automatically detects Oh My Zsh or Oh My Bash and installs the appropriate plugin.

## Manual Installation

### Oh My Zsh

1. Clone or copy the plugin to your custom plugins directory:

```bash
mkdir -p ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/caro
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/plugins/oh-my-zsh/caro.plugin.zsh \
  -o ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/caro/caro.plugin.zsh
```

2. Add `caro` to your plugins in `~/.zshrc`:

```zsh
plugins=(git caro)  # Add caro to your existing plugins
```

3. Restart your terminal or run `source ~/.zshrc`

### Oh My Bash

1. Clone or copy the plugin to your custom plugins directory:

```bash
mkdir -p ${OSH_CUSTOM:-~/.oh-my-bash/custom}/plugins/caro
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/plugins/oh-my-bash/caro.plugin.sh \
  -o ${OSH_CUSTOM:-~/.oh-my-bash/custom}/plugins/caro/caro.plugin.sh
```

2. Add `caro` to your plugins in `~/.bashrc`:

```bash
plugins=(git caro)  # Add caro to your existing plugins
```

3. Restart your terminal or run `source ~/.bashrc`

## What the Plugins Provide

- **PATH Configuration**: Automatically adds `~/.local/bin` and `~/.cargo/bin` to your PATH
- **Shell Completions**: Tab completion for caro commands
- **Aliases**:
  - `c` - shorthand for `caro`
  - `cx` - `caro -x` (execute directly)
  - `ce` - `caro -e` (explain command)
  - `cs` - `caro -s` (safe/strict mode)
- **Functions**:
  - `crun "query"` - Generate command and prompt before executing
  - `cexplain` - Explain your last command in natural language

## Customization

You can override the caro installation directory by setting:

```bash
export CARO_INSTALL_DIR=/custom/path
```
