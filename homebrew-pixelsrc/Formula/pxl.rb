# typed: false
# frozen_string_literal: true

# Homebrew formula for pxl - Pixelsrc CLI
# Install: brew tap scbrown/pixelsrc && brew install pxl
class Pxl < Formula
  desc "GenAI-native pixel art format and compiler"
  homepage "https://github.com/scbrown/pixelsrc"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/scbrown/pixelsrc/releases/download/v#{version}/pxl-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/scbrown/pixelsrc/releases/download/v#{version}/pxl-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/scbrown/pixelsrc/releases/download/v#{version}/pxl-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/scbrown/pixelsrc/releases/download/v#{version}/pxl-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  def install
    bin.install "pxl"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/pxl --version")
  end
end
