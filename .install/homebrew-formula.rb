# This is a template for the Homebrew formula
# For the actual tap, create a separate repository: homebrew-cmdai
# Then place this file as: Formula/cmdai.rb

class Cmdai < Formula
  desc "Convert natural language to safe POSIX shell commands using local LLMs"
  homepage "https://github.com/wildcard/cmdai"
  version "0.1.0"  # This will be updated by release workflow
  license "AGPL-3.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/wildcard/cmdai/releases/download/v#{version}/cmdai-macos-silicon"
      sha256 "PLACEHOLDER_ARM64_SHA256"  # Auto-generated during release
    else
      url "https://github.com/wildcard/cmdai/releases/download/v#{version}/cmdai-macos-intel"
      sha256 "PLACEHOLDER_AMD64_SHA256"  # Auto-generated during release
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/wildcard/cmdai/releases/download/v#{version}/cmdai-linux-arm64"
      sha256 "PLACEHOLDER_LINUX_ARM64_SHA256"  # Auto-generated during release
    else
      url "https://github.com/wildcard/cmdai/releases/download/v#{version}/cmdai-linux-amd64"
      sha256 "PLACEHOLDER_LINUX_AMD64_SHA256"  # Auto-generated during release
    end
  end

  def install
    bin.install "cmdai-macos-silicon" => "cmdai" if Hardware::CPU.arm? && OS.mac?
    bin.install "cmdai-macos-intel" => "cmdai" if Hardware::CPU.intel? && OS.mac?
    bin.install "cmdai-linux-arm64" => "cmdai" if Hardware::CPU.arm? && OS.linux?
    bin.install "cmdai-linux-amd64" => "cmdai" if Hardware::CPU.intel? && OS.linux?

    # Create config directory if needed
    (etc/"cmdai").mkpath

    # Generate shell completions if cmdai supports it
    # generate_completions_from_executable(bin/"cmdai", "completions")
  end

  def caveats
    <<~EOS
      cmdai has been installed! Get started with:
        cmdai --help

      To configure cmdai, edit:
        ~/.config/cmdai/config.toml

      For more information, visit:
        https://github.com/wildcard/cmdai
    EOS
  end

  test do
    assert_match "cmdai", shell_output("#{bin}/cmdai --version")
  end
end
