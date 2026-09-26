cask "spiraler" do
  version "@VERSION@"
  name "Spiraler"
  desc "Local asset studio for coherent visual families"
  homepage "https://github.com/@REPO@"

  on_macos do
    sha256 "@MACOS_SHA256@"
    url "https://github.com/@REPO@/releases/download/v#{version}/Spiraler_#{version}_aarch64.dmg"
    depends_on arch: :arm64
    depends_on cask: "codex"
    app "Spiraler.app"

    caveats <<~EOS
      Spiraler is ad-hoc signed and not notarized. macOS requires approval
      in System Settings > Privacy & Security before the first launch.
      Sign in to Codex to generate images.
    EOS
  end

  on_linux do
    arch arm: "aarch64", intel: "x86_64"
    sha256 arm64_linux: "@ARM64_SHA256@", x86_64_linux: "@X86_64_SHA256@"
    url "https://github.com/@REPO@/releases/download/v#{version}/Spiraler_#{version}_#{arch}.AppImage"
    container type: :naked

    preflight_steps do
      set_permissions "Spiraler_{{version}}_{{arch}}.AppImage", "0755"
    end
    # Extract on launch so the cask also works without a FUSE mount.
    command_wrapper "spiraler",
                    executable: "#{staged_path}/Spiraler_#{version}_#{arch}.AppImage",
                    env: { "APPIMAGE_EXTRACT_AND_RUN" => "1" }

    caveats "Run spiraler to launch. Install the Codex CLI and run codex login to generate images."
  end
end
