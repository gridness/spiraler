# GLib security backport

`glib/` contains the complete published `glib 0.18.5` crate, with the two-line
fix from [gtk-rs-core#1343](https://github.com/gtk-rs/gtk-rs-core/pull/1343)
backported to `src/variant_iter.rs`. The upstream fix commit is
`05dff0ee696f9bcd8617cd48c4b812d046d440cb`.

The original crate was downloaded from
`https://static.crates.io/crates/glib/glib-0.18.5.crate`; its SHA-256 is
`233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5`, matching
the previous Cargo.lock registry checksum. Its upstream source commit is
`42b9caf98e03ded086362d9653ca58fe94dc8658`. Copyright and MIT license files
were preserved. No version number was changed.

[RUSTSEC-2024-0429 / GHSA-wrw7-89jp-8q8g](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)
describes undefined behavior caused by passing an immutable pointer reference
to a variadic GLib function that writes through it. The backport makes the
pointer mutable and passes `&mut p`. Cargo's `[patch.crates-io]` replaces the
registry crate for every transitive consumer, including GTK and WebKit.

Tauri's GTK 3 stack requires GLib 0.18; the published GLib 0.20 fix is outside
that compatible version range. Remove this backport when Tauri's Linux
dependency graph can use a patched registry release. Keep this crate's version
at 0.18.5 until then so its API provenance remains accurate.

The release workflow runs `tests/glib_variant_iter.rs` with optimizations on
both Linux architectures. The tests exercise every iterator method named in
the advisory, including empty and exhausted iterators.
