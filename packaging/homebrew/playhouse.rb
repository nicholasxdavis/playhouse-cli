# frozen_string_literal: true

# Auto-updated by release workflow (scripts/update-homebrew-formula.ts).
# Usage: brew install ./packaging/homebrew/playhouse.rb

class Playhouse < Formula
  desc 'QA CLI for security, functional testing, performance audits, and agent handoff'
  homepage 'https://github.com/nicholasxdavis/playhouse-cli'
  version '0.3.3'
  license 'MIT'

  on_macos do
    on_arm do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.3.3/playhouse-0.3.3-aarch64-apple-darwin.tar.gz'
      sha256 'ddaa147779bcde935dba7c861da2f69434b8029b97bb7239e1d62b23befd4d9a'
    end
    on_intel do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.3.3/playhouse-0.3.3-x86_64-apple-darwin.tar.gz'
      sha256 'dd6842ba6adece57f2db88083274d87cbc695a753db3592670b0626f7c0afca0'
    end
  end

  on_linux do
    on_arm do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.3.3/playhouse-0.3.3-aarch64-unknown-linux-gnu.tar.gz'
      sha256 '3c4d7a56375d27bb5109e82298464665b1abad063a6617fde2030d8813ca2501'
    end
    on_intel do
      url 'https://github.com/nicholasxdavis/playhouse-cli/releases/download/v0.3.3/playhouse-0.3.3-x86_64-unknown-linux-gnu.tar.gz'
      sha256 'e649685dc6faa2650db73393412b0c02538bbf64d90599a41693ca62730effa2'
    end
  end

  def install
    bin.install 'playhouse'
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/playhouse --version")
  end
end
