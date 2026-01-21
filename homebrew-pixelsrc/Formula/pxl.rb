# typed: false
# frozen_string_literal: true

# Homebrew formula for pxl - Pixelsrc CLI
# Install: brew tap scbrown/pixelsrc && brew install pxl
class Pxl < Formula
  desc "GenAI-native pixel art format and compiler"
  homepage "https://github.com/scbrown/pixelsrc"
  version "0.2.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/scbrown/pixelsrc/releases/download/v0.2.0/pxl-v0.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "243cb8a3fddb10ae73c76e9a223ee91fab350d1c76e2e1a6a53d73dd8c5e153b"
    end
    on_intel do
      url "https://github.com/scbrown/pixelsrc/releases/download/v0.2.0/pxl-v0.2.0-x86_64-apple-darwin.tar.gz"
      sha256 "d6f6e559717337b11585a6193f28b2d12afdae46a377f4d0e0385314a053f880"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/scbrown/pixelsrc/releases/download/v0.2.0/pxl-v0.2.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "102572e116fae5e5fcdb2bb75c964eda1d46a66e8304bf34a45d4c3b412cc4d2"
    end
    on_intel do
      url "https://github.com/scbrown/pixelsrc/releases/download/v0.2.0/pxl-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "d376a081598379b105d11a36ef531eaedbb788c447311a643f4a1c190e032893"
    end
  end

  def install
    bin.install "pxl"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/pxl --version")
  end
end
