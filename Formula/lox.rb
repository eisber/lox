class Lox < Formula
  desc "Loxone Miniserver CLI — control lights, blinds, and automations from your terminal"
  homepage "https://github.com/discostu105/lox"
  version "0.4.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-aarch64"
      sha256 "fdcdaa2d040814f3dd65e1f0ff23e280e582b0021b084216364f20a6bfcc4d55"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-x86_64"
      sha256 "248b1c1af26f35b491998f00243bf4ddecc4843c52b01339e49f46388d7c7e24"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-aarch64"
      sha256 "163967856b61a74816afbf0d5a584d260ea84158663d25ca8d0bccfd96010ec3"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-x86_64"
      sha256 "157cb7e7759326b2a7fc634496d03a8f86a135060fe164f4e26aafb425c482d7"
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
