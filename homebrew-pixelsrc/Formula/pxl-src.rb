# typed: false
# frozen_string_literal: true

# Homebrew formula for pxl - build from source
# Install: brew tap stiwi/pixelsrc && brew install pxl-src
class PxlSrc < Formula
  desc "GenAI-native pixel art format and compiler (build from source)"
  homepage "https://github.com/stiwi/pixelsrc"
  url "https://github.com/stiwi/pixelsrc/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256_SOURCE"
  license "MIT"
  head "https://github.com/stiwi/pixelsrc.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/pxl --version")
  end
end
