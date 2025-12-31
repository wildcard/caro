//! Plugin manager and plugin detection
//!
//! Detects installed shell plugin managers (Oh My Zsh, Prezto, Zinit, Fisher)
//! and their enabled plugins.

use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Known shell plugin managers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginManager {
    /// Oh My Zsh framework
    OhMyZsh {
        path: PathBuf,
        plugins: Vec<String>,
        theme: Option<String>,
    },
    /// Prezto framework
    Prezto { path: PathBuf, modules: Vec<String> },
    /// Zinit plugin manager
    Zinit { path: PathBuf, plugins: Vec<String> },
    /// Fisher for Fish shell
    Fisher { path: PathBuf, plugins: Vec<String> },
    /// Antigen plugin manager
    Antigen { plugins: Vec<String> },
    /// Zplug plugin manager
    Zplug { plugins: Vec<String> },
}

impl PluginManager {
    /// Get the name of this plugin manager
    pub fn name(&self) -> &'static str {
        match self {
            Self::OhMyZsh { .. } => "Oh My Zsh",
            Self::Prezto { .. } => "Prezto",
            Self::Zinit { .. } => "Zinit",
            Self::Fisher { .. } => "Fisher",
            Self::Antigen { .. } => "Antigen",
            Self::Zplug { .. } => "Zplug",
        }
    }

    /// Get the list of enabled plugins
    pub fn plugins(&self) -> &[String] {
        match self {
            Self::OhMyZsh { plugins, .. } => plugins,
            Self::Prezto { modules, .. } => modules,
            Self::Zinit { plugins, .. } => plugins,
            Self::Fisher { plugins, .. } => plugins,
            Self::Antigen { plugins } => plugins,
            Self::Zplug { plugins } => plugins,
        }
    }

    /// Check if a specific plugin is enabled
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins().iter().any(|p| p == name)
    }
}

/// Plugin detection service
pub struct PluginDetector {
    home_dir: PathBuf,
}

impl PluginDetector {
    /// Create a new plugin detector
    pub fn new() -> Option<Self> {
        Some(Self {
            home_dir: dirs::home_dir()?,
        })
    }

    /// Create with a custom home directory (for testing)
    pub fn with_home(home: PathBuf) -> Self {
        Self { home_dir: home }
    }

    /// Detect all installed plugin managers
    pub fn detect_all(&self) -> Vec<PluginManager> {
        let mut managers = Vec::new();

        if let Some(omz) = self.detect_ohmyzsh() {
            managers.push(omz);
        }
        if let Some(prezto) = self.detect_prezto() {
            managers.push(prezto);
        }
        if let Some(zinit) = self.detect_zinit() {
            managers.push(zinit);
        }
        if let Some(fisher) = self.detect_fisher() {
            managers.push(fisher);
        }

        managers
    }

    /// Detect Oh My Zsh installation
    pub fn detect_ohmyzsh(&self) -> Option<PluginManager> {
        // Check ZSH env var first
        let omz_path = std::env::var("ZSH")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home_dir.join(".oh-my-zsh"));

        if !omz_path.join("oh-my-zsh.sh").exists() {
            return None;
        }

        // Parse plugins from .zshrc
        let zshrc = self.home_dir.join(".zshrc");
        let (plugins, theme) = if zshrc.exists() {
            self.parse_ohmyzsh_config(&zshrc)
        } else {
            (Vec::new(), None)
        };

        Some(PluginManager::OhMyZsh {
            path: omz_path,
            plugins,
            theme,
        })
    }

    /// Parse Oh My Zsh configuration from .zshrc
    fn parse_ohmyzsh_config(&self, zshrc: &Path) -> (Vec<String>, Option<String>) {
        let content = match std::fs::read_to_string(zshrc) {
            Ok(c) => c,
            Err(_) => return (Vec::new(), None),
        };

        let plugins = self.extract_omz_plugins(&content);
        let theme = self.extract_omz_theme(&content);

        (plugins, theme)
    }

    /// Extract plugins from Oh My Zsh config
    fn extract_omz_plugins(&self, content: &str) -> Vec<String> {
        // Match plugins=(plugin1 plugin2 plugin3) - handles multiline
        let plugins_re =
            Regex::new(r"(?s)plugins=\(\s*([^)]+)\s*\)").expect("Invalid plugins regex");

        if let Some(caps) = plugins_re.captures(content) {
            if let Some(plugins_str) = caps.get(1) {
                return plugins_str
                    .as_str()
                    .split_whitespace()
                    .filter(|s| !s.is_empty() && !s.starts_with('#'))
                    .map(|s| s.trim().to_string())
                    .collect();
            }
        }

        Vec::new()
    }

    /// Extract theme from Oh My Zsh config
    fn extract_omz_theme(&self, content: &str) -> Option<String> {
        let theme_re =
            Regex::new(r#"ZSH_THEME=["']?([^"'\s]+)["']?"#).expect("Invalid theme regex");

        theme_re.captures(content).and_then(|caps| {
            caps.get(1)
                .map(|m| m.as_str().to_string())
                .filter(|s| !s.is_empty())
        })
    }

    /// Detect Prezto installation
    pub fn detect_prezto(&self) -> Option<PluginManager> {
        let prezto_path = self.home_dir.join(".zprezto");

        if !prezto_path.join("init.zsh").exists() {
            return None;
        }

        // Parse modules from .zpreztorc
        let zpreztorc = self.home_dir.join(".zpreztorc");
        let modules = if zpreztorc.exists() {
            self.parse_prezto_modules(&zpreztorc)
        } else {
            Vec::new()
        };

        Some(PluginManager::Prezto {
            path: prezto_path,
            modules,
        })
    }

    /// Parse Prezto modules from .zpreztorc
    fn parse_prezto_modules(&self, zpreztorc: &Path) -> Vec<String> {
        let content = match std::fs::read_to_string(zpreztorc) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        // Match zstyle ':prezto:load' pmodule 'module1' 'module2'
        let modules_re = Regex::new(r"zstyle\s+':prezto:load'\s+pmodule\s+(.+)")
            .expect("Invalid prezto modules regex");

        if let Some(caps) = modules_re.captures(&content) {
            if let Some(modules_str) = caps.get(1) {
                let quote_re = Regex::new(r"'([^']+)'").expect("Invalid quote regex");
                return quote_re
                    .captures_iter(modules_str.as_str())
                    .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                    .collect();
            }
        }

        Vec::new()
    }

    /// Detect Zinit installation
    pub fn detect_zinit(&self) -> Option<PluginManager> {
        let zinit_path = self.home_dir.join(".zinit");
        let zi_path = self.home_dir.join(".zi");

        let path = if zinit_path.exists() {
            zinit_path
        } else if zi_path.exists() {
            zi_path
        } else {
            return None;
        };

        // Parse plugins from .zshrc (zinit light/load commands)
        let zshrc = self.home_dir.join(".zshrc");
        let plugins = if zshrc.exists() {
            self.parse_zinit_plugins(&zshrc)
        } else {
            Vec::new()
        };

        Some(PluginManager::Zinit { path, plugins })
    }

    /// Parse Zinit plugins from .zshrc
    fn parse_zinit_plugins(&self, zshrc: &Path) -> Vec<String> {
        let content = match std::fs::read_to_string(zshrc) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut plugins = HashSet::new();

        // Match zinit light/load commands
        let zinit_re =
            Regex::new(r"zinit\s+(?:light|load)\s+([^\s]+)").expect("Invalid zinit regex");

        for caps in zinit_re.captures_iter(&content) {
            if let Some(plugin) = caps.get(1) {
                plugins.insert(plugin.as_str().to_string());
            }
        }

        plugins.into_iter().collect()
    }

    /// Detect Fisher installation (Fish shell)
    pub fn detect_fisher(&self) -> Option<PluginManager> {
        let fisher_path = self.home_dir.join(".config/fish/functions/fisher.fish");

        if !fisher_path.exists() {
            return None;
        }

        // Parse plugins from fish_plugins file
        let plugins_file = self.home_dir.join(".config/fish/fish_plugins");
        let plugins = if plugins_file.exists() {
            self.parse_fisher_plugins(&plugins_file)
        } else {
            Vec::new()
        };

        Some(PluginManager::Fisher {
            path: fisher_path,
            plugins,
        })
    }

    /// Parse Fisher plugins from fish_plugins file
    fn parse_fisher_plugins(&self, plugins_file: &Path) -> Vec<String> {
        match std::fs::read_to_string(plugins_file) {
            Ok(content) => content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .map(|l| l.trim().to_string())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Get list of aliases provided by a specific Oh My Zsh plugin
    pub fn get_omz_plugin_aliases(&self, plugin: &str) -> Vec<(&'static str, &'static str)> {
        // Return known aliases for common Oh My Zsh plugins
        match plugin {
            "git" => vec![
                ("g", "git"),
                ("ga", "git add"),
                ("gaa", "git add --all"),
                ("gapa", "git add --patch"),
                ("gau", "git add --update"),
                ("gb", "git branch"),
                ("gba", "git branch --all"),
                ("gbd", "git branch --delete"),
                ("gbD", "git branch --delete --force"),
                ("gbl", "git blame -w"),
                ("gbnm", "git branch --no-merged"),
                ("gbr", "git branch --remote"),
                ("gbs", "git bisect"),
                ("gbsb", "git bisect bad"),
                ("gbsg", "git bisect good"),
                ("gbsr", "git bisect reset"),
                ("gbss", "git bisect start"),
                ("gc", "git commit --verbose"),
                ("gc!", "git commit --verbose --amend"),
                ("gca", "git commit --verbose --all"),
                ("gca!", "git commit --verbose --all --amend"),
                ("gcam", "git commit --all --message"),
                ("gcb", "git checkout -b"),
                ("gcd", "git checkout develop"),
                ("gcl", "git clone --recursive"),
                ("gcm", "git checkout main"),
                ("gcmsg", "git commit --message"),
                ("gco", "git checkout"),
                ("gcp", "git cherry-pick"),
                ("gd", "git diff"),
                ("gdca", "git diff --cached"),
                ("gf", "git fetch"),
                ("gfa", "git fetch --all --prune"),
                ("gfo", "git fetch origin"),
                ("gg", "git gui citool"),
                ("ggsup", "git branch --set-upstream-to=origin/$(git_current_branch)"),
                ("gl", "git pull"),
                ("glg", "git log --stat"),
                ("glgg", "git log --graph"),
                ("glgga", "git log --graph --decorate --all"),
                ("glo", "git log --oneline --decorate"),
                ("glol", "git log --graph --pretty='%Cred%h%Creset -%C(auto)%d%Creset %s %Cgreen(%ar) %C(bold blue)<%an>%Creset'"),
                ("gm", "git merge"),
                ("gma", "git merge --abort"),
                ("gmom", "git merge origin/main"),
                ("gp", "git push"),
                ("gpd", "git push --dry-run"),
                ("gpf", "git push --force-with-lease"),
                ("gpf!", "git push --force"),
                ("gpr", "git pull --rebase"),
                ("gpu", "git push upstream"),
                ("gpv", "git push --verbose"),
                ("gr", "git remote"),
                ("gra", "git remote add"),
                ("grb", "git rebase"),
                ("grba", "git rebase --abort"),
                ("grbc", "git rebase --continue"),
                ("grbi", "git rebase --interactive"),
                ("grbm", "git rebase main"),
                ("grbo", "git rebase --onto"),
                ("grbs", "git rebase --skip"),
                ("grh", "git reset HEAD"),
                ("grhh", "git reset HEAD --hard"),
                ("grs", "git restore"),
                ("grss", "git restore --source"),
                ("grst", "git restore --staged"),
                ("gru", "git reset --"),
                ("grup", "git remote update"),
                ("grv", "git remote --verbose"),
                ("gs", "git status"),
                ("gsh", "git show"),
                ("gsi", "git submodule init"),
                ("gsps", "git show --pretty=short --show-signature"),
                ("gss", "git status --short"),
                ("gst", "git status"),
                ("gsta", "git stash push"),
                ("gstaa", "git stash apply"),
                ("gstall", "git stash --all"),
                ("gstc", "git stash clear"),
                ("gstd", "git stash drop"),
                ("gstl", "git stash list"),
                ("gstp", "git stash pop"),
                ("gsts", "git stash show --patch"),
                ("gsu", "git submodule update"),
                ("gsw", "git switch"),
                ("gswc", "git switch --create"),
                ("gswm", "git switch main"),
                ("gts", "git tag --sign"),
                ("gtv", "git tag | sort -V"),
                ("gunignore", "git update-index --no-assume-unchanged"),
                ("gup", "git pull --rebase"),
                ("gupv", "git pull --rebase --verbose"),
                ("gwch", "git whatchanged -p --abbrev-commit --pretty=medium"),
            ],
            "docker" => vec![
                ("dbl", "docker build"),
                ("dcin", "docker container inspect"),
                ("dcls", "docker container ls"),
                ("dclsa", "docker container ls --all"),
                ("dib", "docker image build"),
                ("dii", "docker image inspect"),
                ("dils", "docker image ls"),
                ("dipu", "docker image push"),
                ("dirm", "docker image rm"),
                ("dit", "docker image tag"),
                ("dlo", "docker container logs"),
                ("dnc", "docker network create"),
                ("dncn", "docker network connect"),
                ("dndcn", "docker network disconnect"),
                ("dni", "docker network inspect"),
                ("dnls", "docker network ls"),
                ("dnrm", "docker network rm"),
                ("dpo", "docker container port"),
                ("dps", "docker ps"),
                ("dpsa", "docker ps --all"),
                ("dpu", "docker pull"),
                ("dr", "docker container run"),
                ("drit", "docker container run --interactive --tty"),
                ("drm", "docker container rm"),
                ("drm!", "docker container rm --force"),
                ("dst", "docker container start"),
                ("dstp", "docker container stop"),
                ("dtop", "docker top"),
                ("dvi", "docker volume inspect"),
                ("dvls", "docker volume ls"),
                ("dvprune", "docker volume prune"),
                ("dvrm", "docker volume rm"),
                ("dxc", "docker container exec"),
                ("dxcit", "docker container exec --interactive --tty"),
            ],
            "kubectl" => vec![
                ("k", "kubectl"),
                ("ka", "kubectl apply"),
                ("kaf", "kubectl apply -f"),
                ("kca", "kubectl --all-namespaces"),
                ("kccc", "kubectl config current-context"),
                ("kcdc", "kubectl config delete-context"),
                ("kcgc", "kubectl config get-contexts"),
                ("kcn", "kubectl config set-context --current --namespace"),
                ("kcp", "kubectl cp"),
                ("kcsc", "kubectl config set-context"),
                ("kcuc", "kubectl config use-context"),
                ("kdel", "kubectl delete"),
                ("kdelf", "kubectl delete -f"),
                ("kdes", "kubectl describe"),
                ("kdf", "kubectl diff -f"),
                ("ke", "kubectl edit"),
                ("kex", "kubectl exec -it"),
                ("kgd", "kubectl get deployments"),
                ("kgda", "kubectl get deployments --all-namespaces"),
                ("kgdw", "kubectl get deployments --watch"),
                ("kgdwide", "kubectl get deployments -o wide"),
                ("kgn", "kubectl get nodes"),
                ("kgnw", "kubectl get nodes --watch"),
                ("kgp", "kubectl get pods"),
                ("kgpa", "kubectl get pods --all-namespaces"),
                ("kgpall", "kubectl get pods --all-namespaces -o wide"),
                ("kgpl", "kubectl get pods -l"),
                ("kgpw", "kubectl get pods --watch"),
                ("kgs", "kubectl get services"),
                ("kgsa", "kubectl get services --all-namespaces"),
                ("kgsw", "kubectl get services --watch"),
                ("kgswide", "kubectl get services -o wide"),
                ("kl", "kubectl logs"),
                ("klf", "kubectl logs -f"),
                ("kpf", "kubectl port-forward"),
                ("kra", "kubectl rollout restart deployment"),
                ("krd", "kubectl rollout restart deployment"),
                ("krh", "kubectl rollout history"),
                ("krr", "kubectl rollout restart"),
                ("krs", "kubectl rollout status"),
                ("kru", "kubectl rollout undo"),
                ("ksd", "kubectl scale deployment"),
            ],
            _ => Vec::new(),
        }
    }
}

impl Default for PluginDetector {
    fn default() -> Self {
        Self::new().expect("Could not determine home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manager_name() {
        let omz = PluginManager::OhMyZsh {
            path: PathBuf::new(),
            plugins: vec![],
            theme: None,
        };
        assert_eq!(omz.name(), "Oh My Zsh");
    }

    #[test]
    fn test_plugin_manager_has_plugin() {
        let omz = PluginManager::OhMyZsh {
            path: PathBuf::new(),
            plugins: vec!["git".to_string(), "docker".to_string()],
            theme: None,
        };
        assert!(omz.has_plugin("git"));
        assert!(!omz.has_plugin("kubectl"));
    }

    #[test]
    fn test_extract_omz_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let detector = PluginDetector::with_home(temp_dir.path().to_path_buf());

        let content = r#"
plugins=(
  git
  docker
  kubectl
)
"#;
        let plugins = detector.extract_omz_plugins(content);
        assert_eq!(plugins, vec!["git", "docker", "kubectl"]);
    }

    #[test]
    fn test_extract_omz_theme() {
        let temp_dir = TempDir::new().unwrap();
        let detector = PluginDetector::with_home(temp_dir.path().to_path_buf());

        let content = r#"ZSH_THEME="robbyrussell""#;
        let theme = detector.extract_omz_theme(content);
        assert_eq!(theme, Some("robbyrussell".to_string()));
    }

    #[test]
    fn test_get_git_plugin_aliases() {
        let temp_dir = TempDir::new().unwrap();
        let detector = PluginDetector::with_home(temp_dir.path().to_path_buf());

        let aliases = detector.get_omz_plugin_aliases("git");
        assert!(!aliases.is_empty());
        assert!(aliases.iter().any(|(name, _)| *name == "gst"));
    }
}
