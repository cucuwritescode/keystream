class Keystream < Formula
  desc "Plays sounds while you type on your keyboard"
  homepage "https://github.com/cucuwritescode/keystream"
  license "MIT"
  version "0.1.0"

  url "https://github.com/cucuwritescode/keystream/releases/download/v#{version}/keystream"
  sha256 "" # updated on each release

  depends_on :macos

  def install
    bin.install "keystream"
  end

  service do
    run [opt_bin/"keystream", "run", "pentatonic"]
    keep_alive true
  end

  def caveats
    <<~EOS
      keystream requires accessibility permission to capture keyboard input.

      grant permission in:
        system settings > privacy & security > accessibility

      to start the background service:
        brew services start keystream
    EOS
  end

  test do
    assert_match "KEYSTREAM", shell_output("#{bin}/keystream --help")
  end
end
