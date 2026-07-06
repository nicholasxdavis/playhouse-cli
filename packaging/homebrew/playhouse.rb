# Optional Homebrew formula — install from GitHub Releases.
# Usage (custom tap):
#   brew install ./packaging/homebrew/playhouse.rb
# Or host this file in a tap repo and update url/sha256 per release.

class Playhouse < Formula
  desc "QA CLI for security, functional testing, performance audits, and agent handoff"
  homepage "https://github.com/nicholasxdavis/playhouse-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.1.0/playhouse-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_ON_RELEASE"
    end
    on_intel do
      url "https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.1.0/playhouse-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_ON_RELEASE"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.1.0/playhouse-0.1.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_ON_RELEASE"
    end
    on_intel do
      url "https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.1.0/playhouse-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_ON_RELEASE"
    end
  end

  def install
    bin.install "playhouse"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/playhouse --version")
  end
end
