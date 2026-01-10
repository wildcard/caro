# typed: false
# frozen_string_literal: true

# Homebrew formula for caro - AI-powered shell command assistant
class Caro < Formula
  desc "AI-powered shell command assistant that converts natural language to safe POSIX commands"
  homepage "https://github.com/wildcard/caro"
  version "1.1.0"
  license "AGPL-3.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-macos-silicon"
      sha256 "PLACEHOLDER_MACOS_SILICON_SHA256"

      def install
        bin.install "caro-#{version}-macos-silicon" => "caro"
      end
    else
      url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-macos-intel"
      sha256 "PLACEHOLDER_MACOS_INTEL_SHA256"

      def install
        bin.install "caro-#{version}-macos-intel" => "caro"
      end
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-linux-arm64"
      sha256 "PLACEHOLDER_LINUX_ARM64_SHA256"

      def install
        bin.install "caro-#{version}-linux-arm64" => "caro"
      end
    else
      url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-linux-amd64"
      sha256 "PLACEHOLDER_LINUX_AMD64_SHA256"

      def install
        bin.install "caro-#{version}-linux-amd64" => "caro"
      end
    end
  end

  def caveats
    <<~EOS
      To generate shell completions, run:
        caro --completion bash > $(brew --prefix)/etc/bash_completion.d/caro
        caro --completion zsh > $(brew --prefix)/share/zsh/site-functions/_caro
        caro --completion fish > $(brew --prefix)/share/fish/vendor_completions.d/caro.fish

      Configuration file location: ~/.config/caro/config.toml

      For more information, visit: https://github.com/wildcard/caro
    EOS
  end

  test do
    assert_match "caro #{version}", shell_output("#{bin}/caro --version")
  end
end
