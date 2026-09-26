class Spiraler < Formula
  desc "Local asset studio for coherent visual families"
  homepage "https://github.com/@REPO@"
  version "@VERSION@"
  depends_on :linux

  on_arm do
    url "https://github.com/@REPO@/releases/download/v#{version}/Spiraler_#{version}_aarch64.AppImage", using: :nounzip
    sha256 "@ARM64_SHA256@"
  end
  on_intel do
    url "https://github.com/@REPO@/releases/download/v#{version}/Spiraler_#{version}_x86_64.AppImage", using: :nounzip
    sha256 "@X86_64_SHA256@"
  end

  def install
    appimage = Dir["*.AppImage"].fetch(0)
    chmod 0755, appimage
    system "./#{appimage}", "--appimage-extract"
    libexec.install Pathname("squashfs-root").children
    (bin/"spiraler").write_env_script libexec/"AppRun", APPDIR: libexec
    (share/"applications").install Dir[libexec/"usr/share/applications/*.desktop"]
    share.install libexec/"usr/share/icons" if (libexec/"usr/share/icons").directory?
  end

  def caveats
    "Run spiraler to launch. Install the Codex CLI and run codex login to generate images."
  end
end
