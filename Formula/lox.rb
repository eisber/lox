class Lox < Formula
  desc "Loxone Miniserver CLI — control lights, blinds, and automations from your terminal"
  homepage "https://github.com/discostu105/lox"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-aarch64"
      sha256 "af7cbf0bdc7d4c019841de5388fe9a9b2e6516d316e35841f7a2bacdecb7d6e8"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-x86_64"
      sha256 "a136ec9dc00aa7afc0ec5c002fd69b44f642b4bfd3dbaca4c5f9e2c16c832795"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-aarch64"
      sha256 "0be4351705969e7105e6e2f4b09184d71482c0ca442c1f267838c0e5f09c65cd"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-x86_64"
      sha256 "9524e94d7285f1bd43e9378cbd6f718cc58f80c421b9062ac8aaf819b7928587"
    end
  end

  def install
    binary = Dir.glob("lox-*").first || "lox"
    bin.install binary => "lox"
  end

  test do
    assert_match "lox", shell_output("#{bin}/lox --help")
  end
end
