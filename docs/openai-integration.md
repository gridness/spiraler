# OpenAI integration investigation

Checked September 26, 2026 against official OpenAI documentation and the installed Codex CLI 0.157.1. The user's follow-up explicitly authorized Codex if direct ChatGPT subscription integration was unavailable.

## Decision

There is no documented general-purpose third-party image-generation endpoint that consumes an ordinary ChatGPT subscription. The official [image-generation guide for ChatGPT and Codex](https://learn.chatgpt.com/docs/image-generation) sends programmatic image generation to the separately billed Image API. Apps SDK and GPT Actions extend ChatGPT by allowing ChatGPT to call a developer's service; they do not document an outbound subscription image API for a Tauri client.

This is a finding about the documented interfaces, not a claim that OpenAI could never add one.

Use the official Codex runtime as the authorized fallback. [Codex authentication](https://learn.chatgpt.com/docs/auth) explicitly supports ChatGPT sign-in for subscription usage. The [Codex app-server guide](https://learn.chatgpt.com/docs/app-server) describes embedding Codex in another product. Spiraler uses the CLI's documented noninteractive `exec` interface instead of implementing any OpenAI transport or OAuth exchange itself.

## Capabilities and implementation

| Concern                       | Official evidence                                                                                                                                                                  | Spiraler behavior                                                                                                                                                                                 |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Authentication                | ChatGPT sign-in and API keys are distinct methods in [authentication docs](https://learn.chatgpt.com/docs/auth).                                                                   | Enforces `forced_login_method="chatgpt"`, checks CLI sign-in status, removes API key environment variables from child processes.                                                                  |
| Credential storage            | Codex supports OS keyring storage and automatically refreshes its own sign-in.                                                                                                     | New browser sign-ins request keyring storage. Existing sessions remain under Codex control. Spiraler never reads or copies credentials.                                                           |
| Subscription image generation | [Image generation](https://learn.chatgpt.com/docs/image-generation) documents built-in generation and reference images in Codex.                                                   | Runs the official installed CLI, enables its built-in image generation, and requests one raster output per job. A real PNG was produced and decoded in the acceptance test.                       |
| Reference images              | The same guide documents ordered, multi-image references.                                                                                                                          | Copies immutable image references to the isolated job workspace, attaches them in deterministic order, and assigns subject/style roles explicitly.                                                |
| API models                    | The current [API image guide](https://developers.openai.com/api/docs/guides/image-generation) describes GPT Image 2.5 Sunburst and Flare; this is a different integration.         | Does not assume API capabilities map directly to subscription Codex. No image-model identifier is exposed or spoofed. Codex chooses its supported built-in model.                                 |
| Formats and dimensions        | The API guide documents PNG, JPEG, WebP and model-specific sizes. Codex's public image interface is natural language.                                                              | Accepts validated raster output and stores normalized PNGs. Requests square, portrait, or landscape in the prompt; stores actual dimensions. Does not claim exact resolution or alpha guarantees. |
| Upscaling                     | The API guide establishes generation and edits, but no dedicated lossless upscaler was identified.                                                                                 | A separate local 2× Lanczos operation preserves the source and labels the result as interpolation. No ordinary regeneration is called lossless upscaling.                                         |
| Limits                        | [Codex image documentation](https://learn.chatgpt.com/docs/image-generation) states that images use general Codex usage limits and consume them faster than comparable text turns. | Concurrency is one. Transient connection failures retry at most twice after the first attempt. Authentication, moderation, and quota failures require user action rather than infinite retries.   |

## Request and execution boundaries

`GenerationRequest` separates immutable style, subject, reference material, composition/background constraints, optional source result, and the deterministic prompt. Jobs retain the full snapshot; results retain the job request, provider description, actual dimensions, creation time, and original/variant/upscale relationship.

The provider uses `std::process`/Tokio argument arrays and stdin, never shell interpolation. It invokes `codex exec --ignore-user-config --ephemeral --skip-git-repo-check --sandbox workspace-write --json`. The Codex workspace is a newly named job attempt folder. Only references are copied in. The app validates that `output.png` resolves inside that folder, decodes it under memory and dimension limits, and atomically stores an immutable normalized image and thumbnail before publishing Ready.

Codex is still an agent runtime. Its local sandbox is a runtime boundary, not a guarantee of arbitrary hostile-prompt isolation. This is a personal local studio, not a public service for executing untrusted users' prompts. Spiraler does not give the renderer a general shell command, token accessor, arbitrary file writer, or remote origin permission.

Raw agent streams are drained into a bounded memory buffer, classified into user-facing errors, and discarded. They are not saved in project metadata or shown as debug output. Failed and interrupted job directories are preserved for recovery.

## Verified and unverified

Verified: installed CLI detection, an existing ChatGPT session, an actual image generation and local PNG decode through the production provider code. New account sign-in cannot be completed on someone else's behalf during unattended tests; the implementation delegates that flow to `codex login` and never handles its credentials.

Not claimed: a direct ChatGPT subscription API, exact image dimensions, mathematically guaranteed consistency, unlimited batches, lossless AI upscaling, or availability on every plan. No API fallback is silently substituted.
