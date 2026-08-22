<!------------------------------------------------------------------------- -->

# `list-my-licence`

Embed the verbatim licences of a Rust application and its dependencies into
the binary that ships them.

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

The second is what makes it stay true.  Because both come from the same pass, a
dependency whose licence changed cannot reach a release without that committed
file changing too, and a changed file is a reviewable diff rather than a silent
difference.

<!------------------------------------------------------------------------- -->

## Usage

Name the crate twice.  The build half does the work;  the runtime half holds
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

In continuous integration, add `.checking(true)` to make a stale
`THIRDPARTY.md` fail the build instead of being rewritten.

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

<!------------------------------------------------------------------------- -->

## What it does

**Only what ships.**  Normal and build dependencies, at every level, for the
target triple actually being compiled and the features actually enabled.
Dev-dependencies are followed nowhere;  a test-only crate is never distributed.

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
fact — regularly stale, occasionally wrong, sometimes contradicted by the files
beside it.  Every report says what a package *declares*.

**It is not legal advice.**  Consult somebody qualified before relying on any
of this.

<!------------------------------------------------------------------------- -->

## Related work

[`cargo-about`][cargo-about] and [`cargo-bundle-licenses`][bundle] generate
attribution files out of process, the latter with full texts.
[`cargo-deny`][cargo-deny] enforces licence policy.  [`license-fetcher`][fetch]
and [`notalawyer`][notalawyer] embed texts into the binary as this crate does;
the differences are the coverage model above, the failure policy, the drift
check, and a runtime half with no dependencies.

[bundle]: https://github.com/sstadick/cargo-bundle-licenses
[cargo-about]: https://github.com/EmbarkStudios/cargo-about
[cargo-deny]: https://github.com/EmbarkStudios/cargo-deny
[fetch]: https://github.com/WyvernIXTL/license-fetcher
[notalawyer]: https://github.com/arkedge/notalawyer

<!------------------------------------------------------------------------- -->

## Licence

GPL-3.0-or-later.  See [`LICENCE`](LICENCE) for the full text.

<!------------------------------------------------------------------------- -->
