# Magic Link Terminal - Technical Research

**Feature**: 008-magic-link-terminal
**Created**: 2026-01-04
**Status**: Research Phase

---

## 1. Custom URL Protocol Registration

### macOS

**Approach**: Info.plist CFBundleURLTypes

macOS applications can register custom URL schemes by declaring them in their `Info.plist` file. Since Caro is a CLI tool (not a `.app` bundle), we have two options:

**Option A: Create a minimal .app wrapper**
- Create a minimal application bundle in `/Applications/Caro.app` or `~/Applications/Caro.app`
- The app's only purpose is to handle `caro://` URLs and invoke the CLI
- Structure:
  ```
  Caro.app/
  ├── Contents/
  │   ├── Info.plist          # URL scheme declaration
  │   ├── MacOS/
  │   │   └── caro-launcher   # Shell script or small binary
  │   └── Resources/
  │       └── icon.icns       # Optional app icon
  ```
- Info.plist snippet:
  ```xml
  <key>CFBundleURLTypes</key>
  <array>
    <dict>
      <key>CFBundleURLName</key>
      <string>Caro Magic Link</string>
      <key>CFBundleURLSchemes</key>
      <array>
        <string>caro</string>
      </array>
    </dict>
  </array>
  ```

**Option B: Use LSRegisterURL (programmatic)**
- Use `LSRegisterURL` API to register handler without .app bundle
- Less reliable, may not persist across reboots
- Requires calling system framework APIs from Rust

**Recommendation**: Option A is more reliable and standard

### Linux

**Approach**: XDG Desktop Entry + MIME type

Linux uses the XDG specification for URL scheme handling:

1. Create desktop entry file at `~/.local/share/applications/caro-handler.desktop`:
   ```ini
   [Desktop Entry]
   Name=Caro Magic Link Handler
   Exec=/usr/local/bin/caro-handler %u
   Type=Application
   NoDisplay=true
   MimeType=x-scheme-handler/caro;
   ```

2. Register the handler:
   ```bash
   xdg-mime default caro-handler.desktop x-scheme-handler/caro
   ```

3. Update MIME database:
   ```bash
   update-desktop-database ~/.local/share/applications/
   ```

**Compatibility**: Works on GNOME, KDE, XFCE, and most modern desktop environments.

### Windows

**Approach**: Registry-based URL protocol handler

Windows uses registry entries for custom URL protocols:

1. Create registry entries under `HKEY_CURRENT_USER\Software\Classes\caro`:
   ```reg
   [HKEY_CURRENT_USER\Software\Classes\caro]
   @="URL:Caro Protocol"
   "URL Protocol"=""

   [HKEY_CURRENT_USER\Software\Classes\caro\shell]

   [HKEY_CURRENT_USER\Software\Classes\caro\shell\open]

   [HKEY_CURRENT_USER\Software\Classes\caro\shell\open\command]
   @="\"C:\\Program Files\\Caro\\caro-handler.exe\" \"%1\""
   ```

2. Can be registered during installation or via `caro setup` command

**Note**: Using `HKEY_CURRENT_USER` avoids requiring administrator privileges.

---

## 2. Terminal Detection & Launching

### macOS Terminal Applications

| Terminal | Detection | Launch Command |
|----------|-----------|----------------|
| Terminal.app | Always present | `open -a Terminal "script.sh"` or AppleScript |
| iTerm2 | Check `/Applications/iTerm.app` | `open -a iTerm "script.sh"` or AppleScript |
| Alacritty | Check `which alacritty` | `alacritty -e caro ...` |
| Kitty | Check `which kitty` | `kitty -e caro ...` |
| Warp | Check `/Applications/Warp.app` | `open -a Warp` (limited CLI control) |
| Hyper | Check `/Applications/Hyper.app` | `open -a Hyper` |

**AppleScript for Terminal.app** (recommended for control):
```applescript
tell application "Terminal"
    activate
    do script "caro --magic-link 'encoded-command'"
end tell
```

**AppleScript for iTerm2**:
```applescript
tell application "iTerm2"
    create window with default profile
    tell current session of current window
        write text "caro --magic-link 'encoded-command'"
    end tell
end tell
```

### Linux Terminal Applications

| Terminal | Detection | Launch Command |
|----------|-----------|----------------|
| GNOME Terminal | `which gnome-terminal` | `gnome-terminal -- caro ...` |
| Konsole | `which konsole` | `konsole -e caro ...` |
| xfce4-terminal | `which xfce4-terminal` | `xfce4-terminal -e "caro ..."` |
| Alacritty | `which alacritty` | `alacritty -e caro ...` |
| Kitty | `which kitty` | `kitty -e caro ...` |
| xterm | `which xterm` | `xterm -e caro ...` |
| Tilix | `which tilix` | `tilix -e "caro ..."` |

**Default Terminal Detection**:
1. Check `$TERMINAL` environment variable
2. Check `x-terminal-emulator` symlink (Debian/Ubuntu)
3. Check common terminals in order of preference
4. Fall back to `xterm`

### Windows Terminal Applications

| Terminal | Detection | Launch Command |
|----------|-----------|----------------|
| Windows Terminal | Check registry/wt.exe | `wt.exe -d . caro ...` |
| PowerShell | Always present | `powershell -Command "caro ..."` |
| cmd.exe | Always present | `cmd /c caro ...` |
| Git Bash | Check program files | `bash -c "caro ..."` |

**Windows Terminal Profile** (recommended):
```json
{
    "name": "Caro",
    "commandline": "caro --magic-link \"%1\"",
    "icon": "path/to/caro-icon.png"
}
```

---

## 3. URL Parsing & Security

### URL Structure

Proposed URL format:
```
caro://run?cmd=<url-encoded-command>[&source=<source-url>][&title=<description>]
```

Examples:
```
caro://run?cmd=brew%20install%20jq
caro://run?cmd=curl%20-fsSL%20https%3A%2F%2Fexample.com%2Finstall.sh&source=https%3A%2F%2Fexample.com%2Fdocs
caro://run?cmd=npm%20install%20-g%20typescript&title=Install%20TypeScript
```

### Security Considerations

**Shell Injection Prevention**:
1. URL-decode the command parameter
2. DO NOT pass directly to shell
3. Parse as a structured command (not shell string)
4. Re-escape when constructing terminal command

**Dangerous Patterns in URLs**:
- Embedded shell metacharacters: `; | & $() \`\``
- Command substitution attempts
- Newline injection: `%0a` or `%0d`
- Null byte injection: `%00`

**Validation Steps**:
1. Verify URL scheme is exactly `caro`
2. Verify host/path structure matches expected format
3. URL-decode command parameter
4. Check for injection patterns before safety validation
5. Sanitize for terminal invocation

### Maximum URL Length

Browsers and OSes have URL length limits:
- Chrome: ~2MB (but practical limit ~32KB)
- Firefox: ~65KB
- Safari: ~80KB
- IE/Edge Legacy: ~2KB (not relevant for modern use)
- macOS URL handler: ~32KB practical limit
- Linux xdg-open: System-dependent, typically 128KB+

**Recommendation**: Warn for commands > 8KB, hard limit at 32KB

---

## 4. Prerequisite Detection

### Command Parsing for Programs

To detect what programs a command needs, we can:

1. **Tokenize the command**:
   - Split on shell operators: `; | && || & > < >> <<`
   - Handle quoting: `"string"`, `'string'`
   - Identify program names (first token of each pipeline segment)

2. **Check program availability**:
   ```rust
   fn is_program_available(name: &str) -> bool {
       std::process::Command::new("which")
           .arg(name)
           .output()
           .map(|o| o.status.success())
           .unwrap_or(false)
   }
   ```

3. **Common package manager mappings**:
   ```
   brew (macOS): brew install <package>
   apt (Debian/Ubuntu): apt install <package>
   dnf (Fedora): dnf install <package>
   pacman (Arch): pacman -S <package>
   choco (Windows): choco install <package>
   scoop (Windows): scoop install <package>
   ```

### Known Prerequisite Database

Maintain a database of common tools and their installation methods:

```rust
pub struct PrerequisiteInfo {
    pub name: String,
    pub description: String,
    pub installers: HashMap<Platform, Vec<InstallMethod>>,
    pub min_version: Option<Version>,
    pub verify_command: String,
}

// Example entries
jq: {
    description: "Command-line JSON processor",
    installers: {
        macos: ["brew install jq"],
        debian: ["apt install jq"],
        fedora: ["dnf install jq"],
        windows: ["choco install jq", "scoop install jq"],
    },
    verify_command: "jq --version",
}
```

---

## 5. Browser Extension Architecture

### Chrome Extension (Manifest V3)

**manifest.json**:
```json
{
  "manifest_version": 3,
  "name": "Caro - Run Commands Safely",
  "version": "1.0.0",
  "description": "Detect shell commands and run them safely with Caro",
  "permissions": ["activeTab", "scripting"],
  "content_scripts": [{
    "matches": ["<all_urls>"],
    "js": ["content.js"],
    "css": ["styles.css"]
  }],
  "action": {
    "default_popup": "popup.html"
  }
}
```

**Code Block Detection** (content.js):
```javascript
// Detect code blocks with shell commands
const codeBlocks = document.querySelectorAll('pre code, pre, .highlight');

codeBlocks.forEach(block => {
  const text = block.textContent;
  if (isShellCommand(text)) {
    injectCaroButton(block, text);
  }
});

function isShellCommand(text) {
  // Heuristics for detecting shell commands
  const patterns = [
    /^(brew|apt|npm|pip|cargo|yarn|pnpm)\s+install/,
    /^(curl|wget)\s+/,
    /^(git|docker|kubectl|terraform)\s+/,
    /^(cd|ls|mkdir|rm|mv|cp|cat|grep|find)\s+/,
    /^\$\s+/,  // Lines starting with $ prompt
    /^#\s+/,   // Lines starting with # (often root prompt)
  ];
  return patterns.some(p => p.test(text.trim()));
}
```

**WebAssembly Local Analysis** (optional):
- Compile a lightweight Rust analyzer to WASM
- Run safety classification locally in the browser
- No data sent to external servers
- Provides confidence score before user clicks

### Firefox Extension

Similar architecture with WebExtension APIs (compatible with Manifest V3 subset).

### Privacy Considerations

- Extension NEVER sends code to external servers
- All analysis is local (WASM model or heuristics)
- No tracking, no analytics
- User can disable detection per-site
- Clear indication of what data is used

---

## 6. caro.to Redirect Service Architecture

### Purpose

The `caro.to` redirect service solves critical problems that direct `caro://` links cannot:

1. **Installation Detection**: Direct protocol links fail silently if Caro isn't installed
2. **Referrer Tracking**: Custom protocols don't receive HTTP Referer headers
3. **Preflight Safety**: Show users safety analysis before touching their system
4. **Analytics**: Understand usage patterns and conversion rates

### URL Structure

**Short Link Format**:
```
https://caro.to/r/<link_id>#<url-encoded-command>
```

Example:
```
https://caro.to/r/abc123#brew%20install%20jq
```

**Why URL Fragment (#)?**
- Fragment is never sent to server (privacy by design)
- Command stays client-side only
- Server only sees link_id, not the command content
- JavaScript extracts command for preflight and redirect

### Installation Detection

Detecting if `caro://` protocol is registered is challenging. Approaches:

**Option A: Timeout-based Detection**
```javascript
// Try to open caro:// and detect if it worked
const timeout = setTimeout(() => {
  // Protocol not registered - show install page
  showInstallPage();
}, 2000);

window.addEventListener('blur', () => {
  // Browser lost focus = app opened = protocol works
  clearTimeout(timeout);
});

window.location.href = 'caro://ping';
```

**Option B: Local Storage Flag**
- After successful Caro installation, have CLI call back to caro.to to set a cookie/flag
- Check flag on subsequent visits
- Less reliable but simpler

**Option C: Browser Extension Bridge**
- If browser extension is installed, it can reliably detect CLI installation
- Extension communicates installation status to caro.to

**Recommendation**: Combine Option A (timeout) with Option C (extension enhancement)

### Referrer Capture

The HTTP Referer header provides source context:

```javascript
// Server-side (Node.js example)
app.get('/r/:linkId', (req, res) => {
  const referer = req.headers.referer || req.headers.referrer;
  const linkId = req.params.linkId;

  // Log for analytics (without command content)
  logClick({
    linkId,
    refererDomain: referer ? new URL(referer).hostname : null,
    timestamp: Date.now(),
    userAgent: req.headers['user-agent']
  });

  // Render preflight page (client extracts command from fragment)
  res.render('preflight', { linkId, refererDomain });
});
```

### Data Flow

```
User clicks caro.to/r/abc123#brew%20install%20jq
           │
           ▼
   ┌───────────────────────────────────────┐
   │  Server receives /r/abc123            │
   │  • Captures HTTP Referer              │
   │  • Logs click analytics               │
   │  • Returns preflight HTML + WASM      │
   │  • Does NOT see command (in fragment) │
   └───────────────────────────────────────┘
           │
           ▼
   ┌───────────────────────────────────────┐
   │  Client-side JavaScript               │
   │  • Extracts command from fragment     │
   │  • Loads WASM safety validator        │
   │  • Runs preflight check               │
   │  • Renders safety UI                  │
   └───────────────────────────────────────┘
           │
           ▼
   User sees preflight results, clicks "Open in Caro"
           │
           ▼
   Redirect to caro://run?cmd=...&ref=...&link_id=...
```

### Privacy Architecture

**What server sees**:
- Link ID
- Referrer domain (where user came from)
- Timestamp
- User agent (for platform detection)
- IP address (can be anonymized)

**What server NEVER sees**:
- The actual command (stays in URL fragment)
- Any user data
- Command execution results

---

## 7. WebAssembly Preflight Safety Check

### Architecture

Compile Caro's safety validation module to WebAssembly for browser execution:

```
┌─────────────────────────────────────────────────────────┐
│  Rust Safety Validator                                   │
│  (src/safety/mod.rs, src/safety/patterns.rs)            │
│                                                          │
│  Compile with wasm-pack:                                │
│  wasm-pack build --target web --features wasm           │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│  WASM Module (caro_safety.wasm)                         │
│  • Pattern matching                                      │
│  • Risk level calculation                                │
│  • Explanation generation                                │
│  • ~200-500KB gzipped                                   │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│  JavaScript Wrapper                                      │
│                                                          │
│  import init, { validate_command } from './caro_safety';│
│                                                          │
│  await init();                                           │
│  const result = validate_command(command);              │
│  // { risk_level, warnings, explanation, allowed }      │
└─────────────────────────────────────────────────────────┘
```

### WASM Module Interface

```rust
// src/safety/wasm.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PreflightResult {
    pub risk_level: String,      // "safe", "moderate", "high", "critical"
    pub allowed: bool,
    pub explanation: String,
    pub warnings: Vec<String>,
    pub matched_patterns: Vec<String>,
    pub prerequisites: Vec<String>,
}

#[wasm_bindgen]
pub fn validate_command(command: &str) -> PreflightResult {
    let validator = SafetyValidator::new();
    let result = validator.validate(command);

    PreflightResult {
        risk_level: result.risk_level.to_string(),
        allowed: result.allowed,
        explanation: result.explanation,
        warnings: result.warnings,
        matched_patterns: result.matched_patterns,
        prerequisites: detect_prerequisites(command),
    }
}
```

### Prerequisite Detection in Browser

Limited compared to CLI (no PATH access), but can detect:

1. **Common program patterns**:
   ```javascript
   const KNOWN_PROGRAMS = {
     'brew': { name: 'Homebrew', platforms: ['macos'] },
     'apt': { name: 'APT', platforms: ['linux-debian'] },
     'npm': { name: 'Node.js', platforms: ['all'] },
     'cargo': { name: 'Rust', platforms: ['all'] },
     'docker': { name: 'Docker', platforms: ['all'] },
   };

   function detectPrerequisites(command) {
     const programs = parseCommandPrograms(command);
     return programs.map(p => KNOWN_PROGRAMS[p]).filter(Boolean);
   }
   ```

2. **Display as informational** (can't verify installation from browser):
   ```
   ℹ️ This command uses: brew (Homebrew - macOS package manager)
   Caro will verify this is installed when you run the command.
   ```

### UI Components

**Preflight Landing Page Structure**:

```html
<div class="preflight-container">
  <!-- Header -->
  <header>
    <img src="caro-logo.svg" alt="Caro">
    <h1>Command Safety Check</h1>
    <p class="source">From: <strong>tutorial-site.com</strong></p>
  </header>

  <!-- Command Display -->
  <div class="command-block">
    <code>brew install jq</code>
    <button class="copy-btn">Copy</button>
  </div>

  <!-- Safety Analysis -->
  <div class="safety-result safe"> <!-- or: moderate, high, critical -->
    <div class="risk-badge">✅ Safe</div>
    <p class="explanation">
      This command installs jq, a command-line JSON processor,
      using the Homebrew package manager.
    </p>
  </div>

  <!-- Prerequisites -->
  <div class="prerequisites">
    <h3>Requirements</h3>
    <ul>
      <li>
        <span class="icon">📦</span>
        Homebrew (macOS package manager)
        <span class="note">Caro will check this when you proceed</span>
      </li>
    </ul>
  </div>

  <!-- Warnings (if any) -->
  <div class="warnings" style="display: none;">
    <h3>⚠️ Warnings</h3>
    <ul id="warning-list"></ul>
  </div>

  <!-- Actions -->
  <div class="actions">
    <button class="primary" id="open-caro">Open in Caro</button>
    <button class="secondary" id="cancel">Cancel</button>
  </div>

  <!-- Footer -->
  <footer>
    <p>
      Caro will run additional safety checks locally before executing.
      <a href="/learn-more">Learn how Caro keeps you safe</a>
    </p>
  </footer>
</div>
```

### Performance Considerations

| Metric | Target | Notes |
|--------|--------|-------|
| WASM load time | < 500ms | Lazy load after page render |
| Validation time | < 100ms | Pattern matching is fast |
| Total preflight | < 1s | Including UI render |
| WASM bundle size | < 500KB gzipped | Tree-shake unused code |

### Graceful Degradation

If WASM fails to load:
1. Show command without safety analysis
2. Display warning: "Safety check unavailable - Caro will validate locally"
3. Still allow proceeding to Caro
4. Log failure for debugging

---

## 8. Vendor Security Integration (Phase 5+)

### Potential Integration Partners

| Vendor | Service | Use Case |
|--------|---------|----------|
| VirusTotal | URL/Hash scanning | Check URLs in curl commands |
| Google Safe Browsing | URL reputation | Warn about known malicious URLs |
| Cloudflare Radar | Domain reputation | Trust scoring for source domains |
| Custom | Command hash DB | Community-reported dangerous commands |

### Integration Architecture

```rust
pub trait SecurityVendor: Send + Sync {
    async fn check_url(&self, url: &str) -> Result<ThreatAssessment>;
    async fn check_command_hash(&self, hash: &str) -> Result<ThreatAssessment>;
    fn vendor_name(&self) -> &str;
}

pub struct ThreatAssessment {
    pub is_malicious: bool,
    pub confidence: f32,
    pub details: String,
    pub source: String,
}
```

### Caching Strategy

- Cache positive results (safe) for 24 hours
- Cache negative results (threats) for 7 days
- Always allow manual override with confirmation
- Graceful degradation when API unavailable

---

## 9. Cross-Platform Considerations

### Feature Matrix

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| URL Protocol | .app bundle | .desktop file | Registry |
| Terminal Launch | AppleScript/open | xdg-open/direct | wt.exe/cmd |
| Privilege Detection | sudo | sudo/pkexec | UAC |
| Package Managers | brew | apt/dnf/pacman | choco/scoop |
| Shell Detection | $SHELL | $SHELL | Environment |

### Platform-Specific Challenges

**macOS**:
- Gatekeeper may block unsigned apps
- Solution: Code signing or notarization for production

**Linux**:
- Multiple desktop environments with different behaviors
- Solution: Test on GNOME, KDE, and XFCE; provide fallbacks

**Windows**:
- PowerShell execution policy may block scripts
- Solution: Use `cmd` fallback or guide user to adjust policy

---

## 10. Performance Benchmarks (Targets)

| Operation | Target | Notes |
|-----------|--------|-------|
| URL parsing | < 1ms | Pure string/regex operations |
| Protocol handler launch | < 100ms | OS-dependent |
| Terminal window creation | < 500ms | Terminal-dependent |
| Safety validation | < 500ms | Existing Caro pipeline |
| Prerequisite check | < 200ms | PATH lookup + version check |
| Total click-to-prompt | < 2s | End-to-end user experience |

---

## 11. Open Research Questions

1. **Sandboxed Browsers**: How do Safari's restrictions on protocol handlers affect functionality?
2. **WSL Integration**: Can magic links open commands in WSL on Windows?
3. **Remote SSH**: Could magic links work with SSH-connected terminals?
4. **tmux/screen**: Should magic links integrate with multiplexer sessions?
5. **Accessibility**: How do screen readers interact with magic link confirmations?

---

## 12. Prior Art & References

### Similar Projects

- **GitHub CLI (`gh`)**: Uses protocol handlers for `gh://` authentication
- **VSCode**: Uses `vscode://` for opening files from web
- **Raycast**: Uses custom URL schemes for quick actions
- **Warp Terminal**: Has some web integration features
- **Fig (now part of AWS)**: Had command suggestion features

### Relevant Specifications

- [RFC 3986](https://tools.ietf.org/html/rfc3986): URI Generic Syntax
- [XDG MIME Applications](https://specifications.freedesktop.org/mime-apps-spec/latest/): Linux handler registration
- [Apple URL Scheme Reference](https://developer.apple.com/documentation/xcode/defining-a-custom-url-scheme-for-your-app)
- [Windows Protocol Handlers](https://docs.microsoft.com/en-us/previous-versions/windows/internet-explorer/ie-developer/platform-apis/aa767914(v=vs.85))

---

## Conclusion

The Magic Link Terminal feature is technically feasible across all major platforms. The primary implementation challenges are:

1. **Cross-platform URL protocol registration** - Requires different approaches per OS
2. **Terminal detection and launching** - Many terminals, varying APIs
3. **Security at the URL parsing layer** - Critical to prevent injection
4. **caro.to redirect service** - Requires web infrastructure for referrer tracking and installation detection
5. **WebAssembly preflight** - Requires compiling safety validator to WASM

The `caro.to` redirect service is a critical component that enables:
- **Installation onboarding**: Users without Caro see install instructions instead of broken links
- **Referrer tracking**: We know where users are coming from for analytics and trust assessment
- **Preflight safety**: Users see safety analysis in the browser before their terminal opens
- **Privacy by design**: Commands stay in URL fragments, never sent to server

The phased approach in the spec allows for iterative development:
1. **Phase 1**: Core protocol + basic caro.to redirect
2. **Phase 2**: Enhanced UX + preflight WebAssembly
3. **Phase 3**: Web integration tools
4. **Phase 4**: Browser extension
5. **Phase 5**: Vendor security integration
