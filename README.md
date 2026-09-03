<!------------------------------------------------------------------------- -->

# `list-my-licence`

Embed the verbatim licences of a Rust application and its dependencies into
the binary that ships them.

> **Made with Anthropic Claude.**  The implementation and the documentation
> were written by Claude, working to the direction and review of the author,
> with whom every design decision rests.  Each commit names the model in a
> `Co-Authored-By` trailer, so the record is per change rather than only
> here.

> **Depending on this crate makes your own application GPL-3.0-or-later.**
> The runtime half is linked into the binary it serves, and it is
> GPL-3.0-or-later like everything else here, so a program distributed with it
> must be distributed under GPL-3.0-or-later too.  That is deliberate rather
> than an oversight — [Licence](#licence) explains it in full, and
> [Related work](#related-work) lists alternatives under permissive terms,
> for projects which cannot accept it.

<!------------------------------------------------------------------------- -->

## Summary

Many open source licences oblige a distributed application to reproduce their
text word for word.  Doing that by hand does not scale past a handful of
dependencies, and it rots silently as the dependency graph moves:  nothing
fails, nothing warns, and the attribution simply stops matching what is
shipped.

This crate discharges the obligation from the build instead of from memory.  A
build script resolves the graph, finds the licence files each package actually
ships, judges whether they cover what its manifest declares, and writes two
things — an embedded form for the binary to print, and a `THIRDPARTY.md` kept
under version control.

The second is what keeps it honest.  Both come from the same pass, so a
dependency whose licence changed makes `THIRDPARTY.md` change with it, and a
changed file is a reviewable diff rather than a silent difference.  What turns
that into a guarantee is `checking(true)` below:  in continuous integration the
build refuses to proceed while the committed file disagrees with the graph, so
drift cannot be merged rather than merely being visible.

<!------------------------------------------------------------------------- -->

## Usage

Name the crate twice.  The build half does the work; the runtime half holds
the result and **has no dependencies at all**.

```toml
[build-dependencies]
list-my-licence = { version = "0.1", features = ["build"] }

[dependencies]
list-my-licence = "0.1"
```

A complete `build.rs`:

```rust
fn main() {
    list_my_licence::build::Builder::new()
        .publish("THIRDPARTY.md")
        .run()
        .unwrap_or_else(|error| panic!("{error}"));
}
```

And in the application:

```rust
static LICENCES: list_my_licence::Attribution = list_my_licence::embed!();

fn main() {
    print!("{LICENCES}");
}
```

`Attribution` is plain data.  `Display` renders it for a terminal,
`markdown()` for a file, and the packages can be walked directly for anything
else.

### Keeping the committed file honest

Left alone, the build **refreshes** `THIRDPARTY.md` whenever the graph moves.
That is what you want locally:  the change shows up as a diff, to be reviewed
and committed like any other.

In continuous integration you want the opposite — a build that **refuses** to
proceed while the committed file disagrees with the graph, so that a dependency
whose licence changed cannot be merged unnoticed:

```rust
fn main() {
    let checking = std::env::var_os("CI").is_some();

    list_my_licence::build::Builder::new()
        .publish("THIRDPARTY.md")
        .checking(checking)
        .run()
        .unwrap_or_else(|error| panic!("{error}"));
}
```

With `checking(true)` nothing is written:  the build fails if `THIRDPARTY.md`
is missing or would differ, and the fix is to run the build locally, review the
diff, and commit it.

### With clap

The `clap` feature contributes licence reporting to an application's **own**
parser, rather than replacing the call that parses it — which is what lets it
compose with a derived `Parser` instead of fighting it.

```toml
list-my-licence = { version = "0.1", features = ["clap"] }
```

```rust
#[derive(clap::Parser)]
struct Arguments {
    #[command(flatten)]
    licences: list_my_licence::cli::LicenceArgs,
}

let arguments = Arguments::parse();

arguments.licences.handle_and_exit(&LICENCES);
```

That contributes `--licences` and `--licences <CRATE>`.  Applications
preferring `myapp licences` can flatten `cli::LicenceCommand` into their own
subcommand enumeration instead.

The feature is additive and not default:  it costs the runtime half its empty
dependency list.

### Compressed

The `compression` feature stores the licence texts deflated and inflates them
on demand.  Licence texts compress well — they are long, English, and highly
repetitive — so a binary shipping many of them carries considerably less.

```toml
list-my-licence = { version = "0.1", features = ["compression"] }
```

```rust
static LICENCES: list_my_licence::CompressedAttribution =
    list_my_licence::embed_compressed!();

print!("{LICENCES}");
```

The build script calls `embed_compressed` where it would have called `embed`.
Both may be called; the two artefacts describe the same graph.

This is a **parallel path, not a replacement**.  `embed!`, `Attribution` and
`Licence` are untouched, and a build without the feature keeps its plain text
and its empty dependency list.  That is deliberate:  a Cargo feature may not
change a public type, since two crates in one dependency graph disagreeing
about it would break.

Two things follow from the trade.  Printing costs what the compression saved,
so a binary which never shows its licences never pays it; and notices stay
uncompressed, an Apache-2.0 `NOTICE` being a few lines where a licence is tens
of kilobytes.

<!------------------------------------------------------------------------- -->

## What it does

**Only what ships.**  Normal and build dependencies, at every level, for the
target triple actually being compiled and the features actually enabled.
Dev-dependencies are followed nowhere; a test-only crate is never distributed.

**The copy the author distributed**, wherever there is one.  MIT and BSD
require *the* copyright line, which no canonical text carries, so a licence
file beside the manifest always wins over the SPDX list.  Where a package ships
nothing, the canonical text stands in — and the output says which it was.

**One file covering several licences.**  A crate declaring `MIT OR Apache-2.0`
may ship two files named after the two licences, one file holding both, or
nothing at all.  Reporting a missing licence for a crate that plainly ships one
would destroy trust in the output, so the combined case is recognised rather
than mistaken for a gap.

**`NOTICE` files.**  Cargo models no such concept, so an Apache-2.0 §4(d)
notice is invisible to anything that reads only the manifest.  It is reproduced
alongside the licence rather than instead of it.

**Custom licences.**  A copyright holder may write their own terms.  Both an
SPDX `LicenseRef` and Cargo's `license-file`-without-`license` are recognised
and reproduced from the distributed copy.

<!------------------------------------------------------------------------- -->

## What stops a build

Four things, all of them an obligation that cannot be discharged:

* **A licence needing its own copyright line, with no distributed text.**
  The canonical text would carry an empty `Copyright (c) <year> <holders>`
  and satisfy nobody.
* **An expression nothing available satisfies.**  Whatever was declared, it
  is not covered.
* **No licence declared, and none shipped.**  The obligation cannot even be
  identified, so nothing is reproduced and there is nothing to object to.
* **A declaration that cannot be parsed**, even leniently.  The same, by a
  different route.

An `OR` needs only one of its branches, so an undischargeable term matters only
where the expression cannot be satisfied without it.  A crate offering
`MIT OR Apache-2.0` and shipping only the Apache text passes, taking that
branch.

<!------------------------------------------------------------------------- -->

## What it does not do

**It does not discharge copyleft.**  Reproducing a licence discharges the
reproduction obligation and nothing else.  The corresponding source of the
whole work, the right to relink against a modified library, an offer to users
reaching the software over a network — none of that is expressible as text
embedded in a binary.  What this crate does instead is refuse to be silent
about it:  every licence relied upon is classified by how far its obligations
reach, and anything beyond reproduction is reported for a human to act on,
with a source pointer where one can be derived.

**It is not a policy engine.**  Deciding which licences a project will accept
is [`cargo-deny`][cargo-deny]'s work, done well and at scale.

**It is not an oracle.**  A `license` field is a claim its author made, not a
fact:  a package may declare a licence and ship no file to back it.  That is
why coverage is classified rather than assumed, and why every report says what
a package *declares* rather than what it is.

**It is not legal advice.**  Consult somebody qualified before relying on any
of this.

<!------------------------------------------------------------------------- -->

## Related work

[`cargo-about`][cargo-about] and [`cargo-bundle-licenses`][bundle] generate
attribution files out of process, both able to carry the full texts; the
latter's `--check-previous` is the same idea as the drift check here.
[`cargo-deny`][cargo-deny] enforces licence policy.
[`license-fetcher`][fetch], [`license-retriever`][retriever] and
[`notalawyer`][notalawyer] embed texts into the binary as this crate does, and
the differences are the coverage model above and the failure policy.  A runtime
half with no dependencies distinguishes this crate from `license-fetcher`,
which carries three; it does **not** distinguish it from `notalawyer`, whose
runtime crate has none either.

Links point at crates.io rather than at a repository:  a crate can move
between namespaces on a forge, and its registry page cannot.

[bundle]: https://crates.io/crates/cargo-bundle-licenses
[cargo-about]: https://crates.io/crates/cargo-about
[cargo-deny]: https://crates.io/crates/cargo-deny
[fetch]: https://crates.io/crates/license-fetcher
[notalawyer]: https://crates.io/crates/notalawyer
[retriever]: https://crates.io/crates/license-retriever

<!------------------------------------------------------------------------- -->

## Licence

GPL-3.0-or-later, throughout.  See [`LICENCE`](LICENCE) for the full text.

### What that means for your project

Both halves of this crate are GPL-3.0-or-later, and the runtime half is
**linked into the binary you ship**.  Under the GPL that makes your program and
this crate one combined work, so distributing it obliges you to place the whole
under GPL-3.0-or-later and to offer its corresponding source.  Building against
it without distributing the result — a private tool, an internal service you do
not hand to anyone — obliges nothing; the GPL is triggered by distribution.

The build half is equally GPL-3.0-or-later, but it runs in `build.rs` and is
linked into nothing.  What it writes is your dependencies' licence texts and
your own data:  this crate claims nothing over its output, and `THIRDPARTY.md`
is yours, under whatever terms you choose.

**This was a free choice, and it was made deliberately.**  The runtime half has
no dependencies, so nobody else's terms constrained it and a permissive licence
was available.  It is GPL-3.0-or-later because the author works within the GPL
ecosystem and wanted a tool which stays there.  If your project cannot be
GPL-3.0-or-later, [`license-fetcher`][fetch], [`notalawyer`][notalawyer] and
[`cargo-about`][cargo-about] address much of the same problem under permissive
terms, and [Related work](#related-work) says how they differ.

<!-------------------------------------------------------------------------- -->
