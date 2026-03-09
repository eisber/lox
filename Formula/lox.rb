class Lox < Formula
  desc "Loxone Miniserver CLI — control lights, blinds, and automations from your terminal"
  homepage "https://github.com/discostu105/lox"
  version "0.2.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-aarch64"
      sha256 "7f059e419c1bfa899247753f68a4f7d463a7803f2a47a85cae85914a34d8cc2b"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-x86_64"
      sha256 "76900a826d4228847b6296f457b84ad93a252b03a991d7c8e224d28ae211474e"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-aarch64"
      sha256 "12e0c974f5b050ed34234583cd6cb7ecb549e0264b1579b18423a27bad0e4592"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-x86_64"
      sha256 "2184705bb6ce870b2a9773448f7161ae4f2b6498f53e6df75c320cf83dd214d6"
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
