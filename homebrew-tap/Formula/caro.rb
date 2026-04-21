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
      sha256 "d1d00aeb8bda084d70043ae55f9cef7cb38fa84bd89f14df6ebd43ef86841d9c"

      def install
        bin.install "caro-#{version}-macos-silicon" => "caro"
      end
    else
      url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-macos-intel"
      sha256 "6a44ae52a42f831f35538f93a32cf242be9bae7ae80db91a882d8b69169b8c17"

      def install
        bin.install "caro-#{version}-macos-intel" => "caro"
      end
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-linux-arm64"
      sha256 "0bcb0e000c080ed92899fbc35a0463304ed363cfe584c5164cf2c67e3c849f07"

      def install
        bin.install "caro-#{version}-linux-arm64" => "caro"
      end
    else
      url "https://github.com/wildcard/caro/releases/download/v#{version}/caro-#{version}-linux-amd64"
      sha256 "79992d59227d99e31a72a15cf722692cee52ec7de84c2ee27ca4df7dd30f1d1e"

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
