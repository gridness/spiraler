# Spiraler

A local desktop studio for generating coherent collections of assets. Built with Tauri 2, Rust, Svelte 5, strict TypeScript, and Bun.

## Run

Install [Bun](https://bun.sh), [Rust](https://rustup.rs), the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/), and the official [Codex CLI](https://learn.chatgpt.com/docs/cli). Then:

```sh
bun install --frozen-lockfile
bun run desktop
```

Sign in using **Settings → Connect ChatGPT**, or use an existing `codex login` session. The CLI must support built-in image generation. Development and live generation were verified with Codex CLI 0.157.1 on macOS arm64.

Build a local macOS application:

```sh
bun run tauri build --bundles app
```

The bundle is written to `src-tauri/target/release/bundle/macos/Spiraler.app`. It is ad-hoc signed and not notarized. For Windows and Linux, install their Tauri prerequisites and override the bundle target, for example `bun run tauri build --bundles nsis` or `--bundles deb,appimage` on the matching OS. Windows has not been exercised here.

## Releases and Homebrew

Every push to `main` runs [the desktop release workflow](.github/workflows/release.yml). It builds an Apple Silicon DMG and Linux ARM64 and x86_64 packages in parallel on native runners, attaches them to a GitHub release with [automatically generated release notes](https://cli.github.com/manual/gh_release_create), and updates `Casks/spiraler.rb` and `Formula/spiraler.rb` in `gridness/homebrew-oosama`. The workflow can also be run manually on `main`.

The macOS app is ad-hoc signed before packaging. The workflow mounts the finished DMG and verifies its app signature, sealed resources, architecture, and version before publication. This avoids shipping an app with only a linker signature, which macOS reports as damaged. Ad-hoc signing does not notarize the app or establish an Apple-verified publisher identity. See [Tauri's signing documentation](https://v2.tauri.app/distribute/sign/macos/).

Linux downloads include `Spiraler_<version>_amd64.deb` and `Spiraler_<version>_x86_64.AppImage` for x86_64, and `Spiraler_<version>_arm64.deb` and `Spiraler_<version>_aarch64.AppImage` for ARM64. The native builds use Ubuntu 22.04 as their minimum glibc baseline. Install the matching `.deb` with `sudo apt install ./Spiraler_<version>_<arch>.deb`, or run `chmod +x` on the matching AppImage and launch it. Install and sign in to the official Codex CLI to generate images on Linux as well.

Bun downloads and Rust dependencies and build outputs are cached separately for each platform and architecture. Rust release builds use Thin LTO and parallel code generation to reduce compile and link work. Artifact uploads skip recompressing the already-compressed installers. A first build still needs to populate the caches.

Release versions use the major and minor numbers from `src-tauri/tauri.conf.json`, with the workflow run number added to its patch number. For example, base `0.1.0` and run `12` produce `0.1.12`, tag `v0.1.12`, and `Spiraler_0.1.12_aarch64.dmg`. All three builds use the same version. Rerunning a workflow keeps its version, preserves published assets, and adds missing downloads; an older run cannot downgrade the cask.

The workflow uses your GitHub App with client ID `Iv23liat5k1xe9You0z0`, installed in `spiraler` and `homebrew-oosama`. Add its PEM private key as the `APP_PRIVATE_KEY` repository Actions secret in `spiraler`. The app must have **Contents: read and write** permission for `homebrew-oosama`, and the tap must allow it to push to `main`. The workflow uses [GitHub's App token action](https://github.com/actions/create-github-app-token) to mint a token scoped to the tap and revoke it when the job finishes. The tap commit uses that App's bot name and email, and the App authenticates its push. It uses the repository's `GITHUB_TOKEN` to publish Spiraler releases. No Apple signing credentials are required.

Install the cask on macOS or Linux with current Homebrew:

```sh
brew install --cask gridness/oosama/spiraler
```

On Linux, a formula is also available:

```sh
brew install --formula gridness/oosama/spiraler
spiraler
```

Both Linux packages select the ARM64 or x86_64 AppImage and expose the `spiraler` launcher. The formula extracts the AppImage during installation; the cask extracts it on launch. Neither launcher needs FUSE. CI installs and launches both published packages on each Linux architecture under a virtual display. Homebrew supports Linux casks through OS-specific stanzas; see [the cask cookbook](https://docs.brew.sh/Cask-Cookbook).

On macOS, Homebrew also installs the [Codex CLI cask](https://formulae.brew.sh/cask/codex) as a dependency. On Linux or for a direct DMG installation, install the Codex CLI yourself. Sign in as described above to generate images.

The macOS build is ad-hoc signed and not notarized. After trying to open the installed app, approve it in **System Settings → Privacy & Security → Open Anyway** before the first launch. A launch without this approval requires Developer ID signing and Apple notarization, which need Apple Developer credentials.

To verify a macOS release locally:

```sh
bash scripts/verify-macos-dmg.sh /path/to/Spiraler_0.1.12_aarch64.dmg 0.1.12
```

## Subscription integration

Direct third-party access to ChatGPT's image subscription is not documented by OpenAI. Spiraler uses the explicitly authorized fallback: the official Codex CLI with ChatGPT sign-in and its built-in image generator. It never reads browser cookies or Codex authentication files, implements private OpenAI endpoints, or silently switches to API billing.

Real images were generated through the provider acceptance test and the native app, including a reference-guided generation and a source-guided variation. See [the capability investigation](docs/openai-integration.md) for sources, boundaries, and limitations. The provider is replaceable without changing project storage or the interface.

## Workflow

1. Create a project and describe its style. Import up to five references or promote a finished result to a reference.
2. Add assets one per line. `Name — description`, TXT, headered CSV, and TSV are supported.
3. Select assets and generate. The durable queue runs one job at a time and captures the style and references at enqueue time.
4. Review results in the inspector, choose preferred versions, compare variations, and approve assets.
5. Queue 2× local enlargement if needed, then export selected, approved, or all assets as ordinary PNGs with optional generation metadata.

Upscale uses Lanczos interpolation, not an unverified AI upscaling service. Original files remain intact. Dimensions and transparency requested through Codex are guidance rather than enforced image-model parameters; actual output dimensions are recorded.

## Local files and recovery

Projects live in the operating system's application-data directory under `studio.spiraler.desktop/projects`. On macOS: `~/Library/Application Support/studio.spiraler.desktop/projects`.

Each project contains `project.json`, a previous `project.backup.json`, immutable `images/`, small `thumbnails/`, and `jobs/` with exact requests and provider output. Metadata uses an explicit versioned schema. A newer schema is rejected without rewriting it. Atomic saves sync a temporary file before replacement.

Click the Spiraler logo to see all collections. Delete a collection from that view or its selector. Deletion cancels its work and hides the whole collection; a durable `.deleted` marker keeps its files recoverable. Use **Deleted collections → Restore** to bring it back with its queue paused.

After restart, jobs interrupted during generation become failed and the remaining queue is paused. Retry is explicit because a remote image request may have completed before the app closed. Removing an asset removes its catalog entry; files stay on disk for recovery. Back up the project library with your other creative work.

## Verification

```sh
bun run check
bun test src
python3 -m unittest discover -s scripts -p 'test_*.py'
bun run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
bun run tauri build --bundles app
```

The live provider test consumes subscription usage and is deliberately separate:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --example provider_smoke -- /tmp/spiraler-provider-test
```

See [architecture](docs/architecture.md) and [verification notes](docs/verification.md).
