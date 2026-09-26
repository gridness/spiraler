# Verification

Development host: macOS arm64. Bun 1.4.2, Rust 1.97.1, Codex CLI 0.157.1. Checks dated September 26, 2026.

The release `.app` built successfully. TypeScript/Svelte checks reported zero errors and warnings; all seven frontend tests and twelve Rust integration tests passed. Rust formatting and Clippy (`--all-targets -- -D warnings`) passed.

## Automated coverage

Frontend tests cover plain-line, quoted multiline CSV and TSV imports; malformed and oversized lists; range selection; filtering and failed-job review status.

Rust integration tests cover immutable queued style/subject snapshots; persistent restart recovery; cancellation/retry and duplicate enqueues; atomic backup manifests; corrupt/future schemas left untouched; ordered source/reference roles; safe image import and staged export; path traversal and symlink rejection; and sanitized provider error classification.

The provider acceptance example generated a real PNG using the same `CodexProvider` implementation as the app. It did not use a stub or an API key. The first sandboxed test was blocked from starting Codex's local runtime; the approved host run succeeded.

## Desktop acceptance

The packaged Tauri application was inspected and operated through its native window, not a simulated browser backend. Completed checks:

- Created a project, edited its style, imported a reference through the native picker, and entered eight assets in one paste.
- Selected all eight assets and appended a shared instruction; verified the saved descriptions.
- Generated a real 1,254 × 1,254 image using the reference and the existing ChatGPT-signed-in Codex session, then generated a source-guided variation. Continued using the collection while jobs ran.
- Chose preferred results, approved a result, and produced a 2,508 × 2,508 local enlargement while preserving the original.
- Exported the approved enlarged PNG and metadata through the native folder picker; checked the exported PNG dimensions.
- Quit the debug app and launched the release bundle. All eight assets, the style reference, preferred variation, and three-result history reopened intact.
- Verified consecutive style-field edits after fixing an asynchronous save race. Palette, material, and lighting edits persisted without losing characters.
- Inspected native window layouts at the minimum 900 × 640 size and a larger desktop size. Checked arrow-key asset navigation, Space selection, and the inspector keyboard shortcut.
- Imported a temporary 4,097-pixel-wide fixture and attempted local enlargement. The app rejected it with a clear size-limit message while preserving its source. Removed that test asset through the confirmation dialog; the eight-asset collection remained intact.
- Scanned saved project metadata for credential keys; none were present. Raw provider streams are not persisted.

Failed-job recovery, cancellation, bounded provider-error classification, and interrupted-queue restart behavior were exercised by automated tests. An actual service outage or exhausted subscription was not forced during desktop acceptance.

## Collection navigation and deletion

Verified in the packaged app: the logo opens all collections; the top-bar collection name returns from Style to Assets; the inspector control appears for one selected asset, stays available when collapsed, and disappears for no selection, multiple selections, and other workspaces. A disposable collection was deleted from the collections view, restored, and deleted again through the selector while a different collection stayed open. The release app reopened with the user's original collection selected.

New Rust tests verify durable deletion and restoration across restarts, retained image bytes, cancellation of restored queued/running jobs, preservation of an unrelated selected collection, rejection of unknown IDs, and rejection of a replaced symlink directory. Deleted collections retain their local files and can be restored from the overview.

Use these checks for future releases:

- Create a project, set a style, add references, and enter several asset descriptions in one paste.
- Generate one asset; leave Queue while it runs and edit other assets.
- Choose a result, approve it, generate a variation, compare versions, and use one as a reference.
- Select multiple assets and apply a shared instruction, duplicate them, reorder, and cancel queued work.
- Enlarge a result locally, confirm twice the original dimensions, and export an approved selection.
- Restart the app and verify the same project, results, references, and selection of preferred images remain.
- Pause a queue, quit, relaunch, and verify no background work restarts without confirmation.
- Exercise missing files, invalid images, failed jobs, offline errors, and the smallest supported window size.

## Release limits

Only macOS was executed here. Windows/Linux platform builds, signing, notarization, distribution installers, and a fresh interactive account login require their respective environments. The app does not claim those checks passed. The project has no CI credentials or remote deployment infrastructure.
