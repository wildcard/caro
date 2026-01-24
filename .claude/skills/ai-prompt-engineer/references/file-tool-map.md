# File-to-Tool Mapping

A comprehensive mapping from file types to appropriate tools, organized by platform and use case.

## Core Principle

> **The file extension is the strongest signal for tool selection.**

When a user mentions or references a file, the extension should immediately constrain the tool space. This prevents common errors like suggesting `tar` for `.zip` files or `unzip` for `.tar.gz` files.

---

## Archive Formats

Archives are the canonical example of file-type-driven tool selection. Each format has specific tools, and platform conventions matter.

### Extraction Matrix

| Extension | Format | Linux | macOS | Windows |
|-----------|--------|-------|-------|---------|
| `.tar` | Tar archive | `tar -xf` | `tar -xf` | `tar -xf` (Git Bash) |
| `.tar.gz`, `.tgz` | Gzipped tar | `tar -xzf` | `tar -xzf` | `tar -xzf` |
| `.tar.bz2`, `.tbz2` | Bzip2 tar | `tar -xjf` | `tar -xjf` | `tar -xjf` |
| `.tar.xz`, `.txz` | XZ tar | `tar -xJf` | `tar -xJf` | `tar -xJf` |
| `.tar.zst` | Zstd tar | `tar --zstd -xf` | `tar --zstd -xf` | N/A |
| `.zip` | ZIP archive | `unzip` | `unzip` | `Expand-Archive` / `7z x` |
| `.7z` | 7-Zip | `7z x` | `7z x` | `7z x` |
| `.rar` | RAR archive | `unrar x` | `unrar x` | `unrar x` / WinRAR |
| `.gz` | Gzip (single) | `gunzip` / `gzip -d` | `gunzip` | `gzip -d` |
| `.bz2` | Bzip2 (single) | `bunzip2` | `bunzip2` | `bunzip2` |
| `.xz` | XZ (single) | `unxz` | `unxz` | `unxz` |
| `.Z` | Compress | `uncompress` | `uncompress` | N/A |
| `.lz` | Lzip | `lzip -d` | `lzip -d` | N/A |
| `.lz4` | LZ4 | `lz4 -d` | `lz4 -d` | N/A |
| `.zst` | Zstd | `zstd -d` | `zstd -d` | N/A |

### Compression Matrix (Creating Archives)

| Target Format | Linux | macOS | Windows |
|---------------|-------|-------|---------|
| `.tar.gz` | `tar -czf out.tar.gz dir/` | `tar -czf out.tar.gz dir/` | `tar -czf` |
| `.tar.bz2` | `tar -cjf out.tar.bz2 dir/` | `tar -cjf out.tar.bz2 dir/` | `tar -cjf` |
| `.tar.xz` | `tar -cJf out.tar.xz dir/` | `tar -cJf out.tar.xz dir/` | `tar -cJf` |
| `.zip` | `zip -r out.zip dir/` | `zip -r out.zip dir/` | `Compress-Archive` |
| `.7z` | `7z a out.7z dir/` | `7z a out.7z dir/` | `7z a out.7z dir/` |
| `.gz` (single) | `gzip file` | `gzip file` | `gzip file` |

### Platform Conventions

**Linux:**
- Default distribution format: `.tar.gz` (source), `.deb`/`.rpm` (packages)
- `tar` is always available, handles most formats
- `unzip` usually available, may need install on minimal systems
- 7-zip via `p7zip` package

**macOS:**
- Finder prefers `.zip` for sharing
- `tar` and `unzip` built-in
- 7-zip via `brew install p7zip`
- System downloads often `.dmg` (disk image)

**Windows:**
- Native ZIP support in Explorer and PowerShell
- `.7z` popular, requires 7-Zip installation
- Git Bash provides Unix-like `tar`, `gzip`, `unzip`
- PowerShell: `Expand-Archive`, `Compress-Archive`

### Decision Tree for Extraction

```
Has file extension?
├── Yes
│   ├── .tar.* → tar (with appropriate flag)
│   ├── .zip → unzip (Mac/Linux) or Expand-Archive (Windows)
│   ├── .7z → 7z x
│   ├── .rar → unrar x
│   └── .gz/.bz2/.xz (alone) → gunzip/bunzip2/unxz
└── No extension
    └── Check with `file` command, then apply above
```

---

## Source Code Files

### Language Detection and Tooling

| Extension | Language | Compile | Run | Format | Lint |
|-----------|----------|---------|-----|--------|------|
| `.rs` | Rust | `cargo build` | `cargo run` | `rustfmt` | `clippy` |
| `.go` | Go | `go build` | `go run` | `gofmt` | `golint` |
| `.py` | Python | N/A | `python` | `black` | `pylint`, `ruff` |
| `.js` | JavaScript | N/A | `node` | `prettier` | `eslint` |
| `.ts` | TypeScript | `tsc` | `ts-node` | `prettier` | `eslint` |
| `.c` | C | `gcc` / `clang` | direct | `clang-format` | `cppcheck` |
| `.cpp`, `.cc` | C++ | `g++` / `clang++` | direct | `clang-format` | `cppcheck` |
| `.h`, `.hpp` | C/C++ Header | N/A | N/A | `clang-format` | N/A |
| `.java` | Java | `javac` | `java` | `google-java-format` | `checkstyle` |
| `.rb` | Ruby | N/A | `ruby` | `rubocop` | `rubocop` |
| `.php` | PHP | N/A | `php` | `php-cs-fixer` | `phpstan` |
| `.swift` | Swift | `swiftc` | `swift` | `swift-format` | `swiftlint` |
| `.kt` | Kotlin | `kotlinc` | `kotlin` | `ktlint` | `ktlint` |
| `.scala` | Scala | `scalac` | `scala` | `scalafmt` | `scalafix` |
| `.ex`, `.exs` | Elixir | `mix compile` | `elixir` | `mix format` | `credo` |
| `.erl` | Erlang | `erlc` | `erl` | N/A | `dialyzer` |
| `.hs` | Haskell | `ghc` | `runhaskell` | `ormolu` | `hlint` |
| `.ml` | OCaml | `ocamlopt` | `ocaml` | `ocamlformat` | N/A |
| `.lua` | Lua | N/A | `lua` | `stylua` | `luacheck` |
| `.pl` | Perl | N/A | `perl` | `perltidy` | `perlcritic` |

### Shell Scripts

| Extension | Shell | Run | Lint |
|-----------|-------|-----|------|
| `.sh` | POSIX Shell | `sh` / `bash` | `shellcheck` |
| `.bash` | Bash | `bash` | `shellcheck` |
| `.zsh` | Zsh | `zsh` | `shellcheck` |
| `.fish` | Fish | `fish` | N/A |
| `.ps1` | PowerShell | `pwsh` | `PSScriptAnalyzer` |
| `.bat`, `.cmd` | Batch | `cmd /c` | N/A |

---

## Configuration Files

### Detection and Handling

| Extension/Name | Format | View | Edit | Validate |
|----------------|--------|------|------|----------|
| `.json` | JSON | `jq .` | `jq` | `jq .` / `jsonlint` |
| `.yaml`, `.yml` | YAML | `yq .` | `yq` | `yamllint` |
| `.toml` | TOML | `cat` | editor | `taplo check` |
| `.xml` | XML | `xmllint --format` | editor | `xmllint` |
| `.ini`, `.cfg` | INI | `cat` | editor | N/A |
| `.env` | Env vars | `cat` | editor | `dotenv-linter` |
| `.conf` | Various | `cat` | editor | tool-specific |
| `Makefile` | Make | `cat` | editor | `make -n` |
| `Dockerfile` | Docker | `cat` | editor | `hadolint` |
| `.gitignore` | Gitignore | `cat` | editor | N/A |

### Project Files (Indicate Toolchain)

| File | Project Type | Build Command | Test Command |
|------|--------------|---------------|--------------|
| `Cargo.toml` | Rust | `cargo build` | `cargo test` |
| `package.json` | Node.js | `npm run build` | `npm test` |
| `go.mod` | Go | `go build` | `go test` |
| `pyproject.toml` | Python | `poetry build` | `pytest` |
| `pom.xml` | Maven/Java | `mvn package` | `mvn test` |
| `build.gradle` | Gradle | `gradle build` | `gradle test` |
| `CMakeLists.txt` | CMake | `cmake --build` | `ctest` |
| `Gemfile` | Ruby | `bundle install` | `rake test` |
| `composer.json` | PHP | `composer install` | `phpunit` |
| `mix.exs` | Elixir | `mix compile` | `mix test` |

---

## Data Files

### Reading and Processing

| Extension | Format | View | Query/Process | Convert |
|-----------|--------|------|---------------|---------|
| `.csv` | CSV | `cat`, `less` | `csvq`, `xsv`, `awk` | `csvtool` |
| `.tsv` | TSV | `cat`, `less` | `awk`, `cut` | `csvtool` |
| `.json` | JSON | `jq .` | `jq` | `jq` |
| `.jsonl` | JSON Lines | `jq -c .` | `jq -s` | `jq` |
| `.xml` | XML | `xmllint --format` | `xmlstarlet` | `xmlstarlet` |
| `.parquet` | Parquet | `parquet-tools` | `duckdb` | `arrow` |
| `.sqlite`, `.db` | SQLite | `sqlite3` | `sqlite3` | `sqlite3` |

### Binary Data

| Extension | Type | View | Convert |
|-----------|------|------|---------|
| `.bin` | Binary | `xxd`, `hexdump` | N/A |
| `.hex` | Hex dump | `xxd -r` | `xxd -r` |

---

## Document Files

### Reading and Conversion

| Extension | Format | View | Convert To |
|-----------|--------|------|------------|
| `.md` | Markdown | `cat`, `glow` | `pandoc` → HTML/PDF |
| `.rst` | reStructuredText | `cat` | `pandoc` |
| `.txt` | Plain text | `cat`, `less` | N/A |
| `.pdf` | PDF | `pdftotext` (extract) | `pandoc`, `convert` |
| `.doc`, `.docx` | Word | `pandoc` (extract) | `pandoc`, `libreoffice` |
| `.html` | HTML | `lynx -dump`, `w3m` | `pandoc` |
| `.tex` | LaTeX | `cat` | `pdflatex`, `pandoc` |

---

## Image Files

### Processing Matrix

| Extension | Format | View (CLI) | View (GUI) | Process | Convert |
|-----------|--------|------------|------------|---------|---------|
| `.png` | PNG | `viu`, `catimg` | `open` (Mac), `xdg-open` | `convert` (ImageMagick) | `convert` |
| `.jpg`, `.jpeg` | JPEG | `viu`, `catimg` | `open`, `xdg-open` | `convert` | `convert` |
| `.gif` | GIF | `viu` | `open` | `convert`, `gifsicle` | `convert` |
| `.svg` | SVG | `cat` (source) | browser | `inkscape` | `convert` |
| `.webp` | WebP | `viu` | `open` | `cwebp`, `dwebp` | `convert` |
| `.heic`, `.heif` | HEIC | N/A | `open` (Mac) | `sips` (Mac) | `sips`, `convert` |
| `.bmp` | Bitmap | `viu` | `open` | `convert` | `convert` |
| `.tiff`, `.tif` | TIFF | N/A | `open` | `convert` | `convert` |
| `.ico` | Icon | N/A | `open` | `convert` | `convert` |
| `.psd` | Photoshop | N/A | N/A | `convert` | `convert` |
| `.raw`, `.cr2`, `.nef` | Camera RAW | N/A | N/A | `dcraw`, `darktable-cli` | `dcraw` |

### Platform-Specific Image Tools

**macOS:**
- `sips` - Native image manipulation
  - Resize: `sips -z height width file.jpg`
  - Convert: `sips -s format png file.jpg --out file.png`
  - HEIC→JPEG: `sips -s format jpeg file.heic --out file.jpg`

**Linux:**
- `convert` (ImageMagick) - Universal converter
  - Resize: `convert file.jpg -resize 800x600 out.jpg`
  - Convert: `convert file.png file.jpg`
- `viu` - Terminal image viewer

**Windows:**
- PowerShell + System.Drawing
- `magick` (ImageMagick)

---

## Audio/Video Files

### Media Processing

| Extension | Type | Info | Convert | Play (CLI) |
|-----------|------|------|---------|------------|
| `.mp3` | Audio | `ffprobe` | `ffmpeg` | `mpv`, `play` |
| `.wav` | Audio | `ffprobe` | `ffmpeg` | `mpv`, `aplay` |
| `.flac` | Audio | `ffprobe` | `ffmpeg` | `mpv` |
| `.ogg` | Audio | `ffprobe` | `ffmpeg` | `mpv` |
| `.m4a`, `.aac` | Audio | `ffprobe` | `ffmpeg` | `mpv` |
| `.mp4` | Video | `ffprobe` | `ffmpeg` | `mpv`, `ffplay` |
| `.mkv` | Video | `ffprobe` | `ffmpeg` | `mpv` |
| `.avi` | Video | `ffprobe` | `ffmpeg` | `mpv` |
| `.mov` | Video | `ffprobe` | `ffmpeg` | `mpv` |
| `.webm` | Video | `ffprobe` | `ffmpeg` | `mpv` |

### Common ffmpeg Operations

```bash
# Convert format
ffmpeg -i input.mp4 output.mkv

# Extract audio
ffmpeg -i video.mp4 -vn -acodec copy audio.m4a

# Resize video
ffmpeg -i input.mp4 -vf scale=1280:720 output.mp4

# Convert to GIF
ffmpeg -i input.mp4 -vf "fps=10,scale=320:-1" output.gif
```

---

## Log and Output Files

### Viewing and Searching

| Extension | Type | View | Search | Monitor |
|-----------|------|------|--------|---------|
| `.log` | Log file | `less`, `cat` | `grep` | `tail -f` |
| `.out` | Output | `less` | `grep` | `tail -f` |
| `.err` | Error log | `less` | `grep` | `tail -f` |
| `.access` | Access log | `less` | `grep`, `goaccess` | `tail -f` |

---

## Package Files

### System-Specific Package Handling

| Extension | System | Install | Info | Extract |
|-----------|--------|---------|------|---------|
| `.deb` | Debian/Ubuntu | `sudo dpkg -i` | `dpkg -I` | `dpkg -x` |
| `.rpm` | RHEL/Fedora | `sudo rpm -i` | `rpm -qip` | `rpm2cpio \| cpio -id` |
| `.apk` | Alpine | `apk add --allow-untrusted` | `apk info` | `tar -xzf` |
| `.pkg` | macOS | `sudo installer -pkg` | `pkgutil --info` | `pkgutil --expand` |
| `.dmg` | macOS | `open` / `hdiutil attach` | `hdiutil info` | `hdiutil attach` |
| `.msi` | Windows | `msiexec /i` | `msiexec /?` | `msiexec /a` |
| `.exe` | Windows | direct / `./` | N/A | N/A |
| `.appimage` | Linux | `chmod +x && ./` | N/A | `--appimage-extract` |
| `.flatpak` | Linux | `flatpak install` | `flatpak info` | N/A |
| `.snap` | Linux | `snap install` | `snap info` | N/A |

---

## Inference Rules

### Extension Priority

When multiple signals conflict, prioritize in this order:

1. **Explicit user mention** - "use tar to extract"
2. **File extension** - `.zip` → `unzip`, not `tar`
3. **Platform default** - Same extension, different tools per OS
4. **Content inspection** - Use `file` command if no extension

### Ambiguity Resolution

**Scenario: No extension**
```bash
# Check actual file type
file mystery_file
# Output: mystery_file: gzip compressed data
# → Use gunzip or tar -xzf depending on content
```

**Scenario: Unknown extension**
```bash
# Check with file command
file data.xyz
# If recognized → use appropriate tool
# If not → inform user, suggest inspection options
```

**Scenario: Multiple files, different types**
```
User: "extract these archives"
Files: data.tar.gz, backup.zip, old.rar

Response: Each archive requires a different tool:
- data.tar.gz → tar -xzf data.tar.gz
- backup.zip → unzip backup.zip
- old.rar → unrar x old.rar
```

### Tool Availability Fallbacks

| Preferred | Fallback 1 | Fallback 2 |
|-----------|------------|------------|
| `jq` | `python -m json.tool` | `cat` |
| `yq` | `python -c "import yaml"` | `cat` |
| `unzip` | `7z x` | `python -m zipfile` |
| `convert` | `sips` (macOS) | inform user |
| `fd` | `find` | `ls -R \| grep` |
| `rg` | `grep -r` | `ack` |
| `bat` | `cat` | `less` |
| `exa`/`eza` | `ls -la` | `ls` |

---

## Quick Reference Cards

### "I want to extract..." Decision Tree

```
What's the file extension?
│
├── .tar, .tar.gz, .tar.bz2, .tar.xz, .tgz, .tbz2, .txz
│   └── tar -x[z|j|J]f filename
│
├── .zip
│   ├── Linux/macOS → unzip filename
│   └── Windows → Expand-Archive filename
│
├── .7z
│   └── 7z x filename (requires 7-zip)
│
├── .rar
│   └── unrar x filename (requires unrar)
│
├── .gz (alone, not .tar.gz)
│   └── gunzip filename OR gzip -d filename
│
├── .bz2 (alone)
│   └── bunzip2 filename
│
├── .xz (alone)
│   └── unxz filename
│
└── No extension
    └── Run: file filename
        └── Then apply appropriate tool
```

### "I want to convert image..." Decision Tree

```
What platform?
│
├── macOS
│   └── sips -s format TARGET input --out output
│       Examples:
│       - HEIC→JPEG: sips -s format jpeg photo.heic --out photo.jpg
│       - PNG→JPEG: sips -s format jpeg image.png --out image.jpg
│       - Resize: sips -Z 1024 image.jpg (max dimension 1024)
│
├── Linux
│   └── convert input output (ImageMagick)
│       Examples:
│       - PNG→JPEG: convert image.png image.jpg
│       - Resize: convert image.jpg -resize 1024x768 resized.jpg
│
└── Windows
    └── magick input output (ImageMagick)
        └── Same syntax as Linux convert
```
