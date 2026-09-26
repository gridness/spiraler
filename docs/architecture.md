# Architecture

## Responsibilities

| Module                      | Responsibility                                                                                               |
| --------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `src/lib/types.ts`          | Strict renderer domain and command unions                                                                    |
| `src/lib/studio.svelte.ts`  | Native command boundary, sequential edits, event reconciliation by revision, connection state, notifications |
| `src/lib/assets.ts`         | List import, filtering, selection ranges, review status                                                      |
| `src/components/`           | Asset grid, inspector, style workspace, queue, export, settings, dialogs                                     |
| `src-tauri/src/domain.rs`   | Typed schema, validation, action reducer, immutable request construction                                     |
| `src-tauri/src/storage.rs`  | Atomic persistence, schema gate, recovery, path confinement, image validation and thumbnails                 |
| `src-tauri/src/provider.rs` | Replaceable `ImageProvider` trait and official Codex adapter                                                 |
| `src-tauri/src/queue.rs`    | One background worker, cancellation, retry delay, processing, publication                                    |
| `src-tauri/src/export.rs`   | Deterministic filenames, result selection, staged batch export                                               |
| `src-tauri/src/lib.rs`      | Narrow Tauri commands and native dialogs                                                                     |

## Persisted schema

Schema 1 stores a project identity, metadata, revision, current Style, style history, ordered assets, results, and jobs. IDs are UUIDs. Every result points to an immutable image and thumbnail. Relative image paths are portable within the project. Moving a whole library is possible through filesystem backup/restore; an in-app arbitrary-project opener is not provided.

```
projects/<project-id>/
  project.json
  project.backup.json
  images/<image-id>.png
  thumbnails/<image-id>.png
  jobs/<job-id>/attempt-<number>-<unique-id>/
    request.json
    reference-1.png
    output.png
```

`preferences.json` stores only the last project ID. The Tauri window-state plugin restores the window. Codex credentials are owned by Codex and its OS credential store, outside project metadata.

## Durability and concurrency

Store mutations happen under a Rust mutex. Each command clones the current project, validates and applies its action, writes and syncs the new manifest through an atomic temporary-file replacement, then publishes the in-memory state. Failed changes leave the previous state intact. The prior manifest is saved independently as a recovery copy.

Image processing runs outside the UI thread. Output and thumbnail are written before the project references them. A crash between those steps leaves recoverable orphan files, not dangling completed results. Deleted assets leave files behind deliberately; there is no automatic garbage collection that could destroy history.

Exports stage every file in a temporary destination folder, then rename the folder only after successful completion. Each batch has a unique directory and each asset name includes an ordinal and stable ID. Export does not overwrite original or previous output.

Queued requests are immutable. Restart marks in-flight jobs failed, pauses queued work, and preserves files. A retry gets a fresh attempt directory so stale output can never be mistaken for a newly generated image. Cancellation changes the persisted state before signalling the worker. Completion checks cancellation again before publishing.

## Security controls

Applied security-guidance requirements include ASVS 1.2.5 for argv-based process calls; 1.5.2 for typed deserialization; 2.2.1 and 2.2.2 for backend validation; 5.2.1, 5.2.2 and 5.2.6 for bounded decoded images; 5.3.2 for generated filenames and canonical path confinement; 13.3.2 for credential isolation; and 15.4.1 for synchronized storage.

Only the main local webview has event/title permissions. Dialogs and reveal operations are implemented in Rust. The renderer cannot invoke a general-purpose shell or filesystem plugin. CSP restricts scripts to bundled code, connections to local IPC, and images to local sources. Svelte escapes text rather than rendering arbitrary HTML.

## Product decisions

Dark charcoal, muted olive, and a warm clay accent keep artwork prominent. The left rail stays stable while the inspector changes with selection. Small windows collapse rail labels and retain accessible button names. Full-resolution images load only in the inspector; the grid lazy-loads bounded 480px thumbnails.

Bulk actions remain visible rather than hiding behind hover menus. Native dialogs handle image/list import and export folders. Keyboard commands cover creation, search, select-all, generation, inspector visibility, and grid navigation. Reduced-motion preferences disable animation. Image quality and consistency remain review decisions; approval always belongs to the user.
