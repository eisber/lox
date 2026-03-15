class Lox < Formula
  desc "Loxone Miniserver CLI — control lights, blinds, and automations from your terminal"
  homepage "https://github.com/discostu105/lox"
  version "0.6.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-aarch64"
      sha256 "7b414cc43ff0b4feb98659a8e92f107e4fd7a87a209576d77ad51532413421c0"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-macos-x86_64"
      sha256 "59080e96f3f6ebb3945616670e374f4b6c0ad64b180f40d8009cdb5269cb6278"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-aarch64"
      sha256 "7166f774b4d0bdeb5c7049b9a471448bfbf756c00cdb5b5d6e30fb8a965bbc72"
    else
      url "https://github.com/discostu105/lox/releases/download/v#{version}/lox-linux-x86_64"
      sha256 "7762bfe8c15e091b06d47af7b8561a1639ea912ff8a5dc8d924a8befc58eecc5"
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
