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

The bundle is written to `src-tauri/target/release/bundle/macos/Spiraler.app`. It is not signed or notarized. For Windows and Linux, install their Tauri prerequisites and override the bundle target, for example `bun run tauri build --bundles nsis` or `--bundles appimage` on the matching OS. Those platforms have not been exercised here.

## Releases and Homebrew

Every push to `main` runs [the macOS release workflow](.github/workflows/release.yml). It builds an unsigned Apple Silicon DMG, attaches it to a GitHub release with [automatically generated release notes](https://cli.github.com/manual/gh_release_create), and updates `Casks/spiraler.rb` in `gridness/homebrew-oosama`. The workflow can also be run manually on `main`.

Release versions use the major and minor numbers from `src-tauri/tauri.conf.json`, with the workflow run number added to its patch number. For example, base `0.1.0` and run `12` produce `0.1.12`, tag `v0.1.12`, and `Spiraler_0.1.12_aarch64.dmg`. The app bundle uses the same version. Rerunning a workflow keeps its version and reuses the published DMG; an older run cannot downgrade the cask.

The workflow uses your GitHub App with client ID `Iv23liat5k1xe9You0z0`, installed in `spiraler` and `homebrew-oosama`. Add its PEM private key as the `APP_PRIVATE_KEY` repository Actions secret in `spiraler`. The app must have **Contents: read and write** permission for `homebrew-oosama`, and the tap must allow it to push to `main`. The workflow uses [GitHub's App token action](https://github.com/actions/create-github-app-token) to mint a token scoped to the tap and revoke it when the job finishes. It uses the repository's `GITHUB_TOKEN` to publish Spiraler releases. No Apple signing credentials are required.

Once the first release and cask update finish, install with:

```sh
brew install --cask gridness/oosama/spiraler
```

Homebrew also installs the [Codex CLI cask](https://formulae.brew.sh/cask/codex) as a dependency. Sign in as described above to generate images. For a direct DMG installation, install the Codex CLI yourself.

The build is unsigned and not notarized. macOS may require approval in **System Settings → Privacy & Security** before the first launch.

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
