recaphub
========

This is a quick helper CLI I wrote to help me track what I've been working on.

### Setup

This tool requires that you setup a personal access token. You can create one by going to `Settings > Developer Settings (this is on the bottom of the sidebar on the left) > Personal Access Tokens`

For convenience this crate uses `dotenv` to allow you to persistently load your personal access token.

To use this, create a `.env` file containing `GITHUB_TOKEN=<your personal access token>`.

### Output

Here's a sample of the kind of output it generates. This output will change
over time as I update it to be more comprehensive with its analysis. Off of the
top of my head I'd like it to also check Zulip threads that I've participated
in and PRs or issues that I've created. Maybe commits as well? I'm not sure,
it's a work in progress. I'm open to suggestions!

```console
❯ cargo run -- --name yaahc --timeframe 7d
- rust-lang/rust: Add #[track_caller] support to `?`
  - https://github.com/rust-lang/rust/issues/77474#issuecomment-1074480790
- rust-lang/rust-clippy: invert check for `clippy::try_err` and disable it inside of try blocks
  - https://github.com/rust-lang/rust-clippy/issues/5757#issuecomment-1069533406
- rust-lang/rust: Tracking issue for `IntoFuture`
  - https://github.com/rust-lang/rust/issues/67644#issuecomment-1074497601
- rust-lang/rust: Unstably constify `impl<I: Iterator> IntoIterator for I`
  - https://github.com/rust-lang/rust/pull/90602#issuecomment-1072618654
  - https://github.com/rust-lang/rust/pull/90602#issuecomment-1073245869
  - https://github.com/rust-lang/rust/pull/90602#issuecomment-1073247688
  - https://github.com/rust-lang/rust/pull/90602#issuecomment-1074491176
- rust-lang/rust: Add ability to spawn Windows process with Proc Thread Attributes
  - https://github.com/rust-lang/rust/pull/88193#issuecomment-1072609119
- rust-lang/rust: add module-level documentation for vec's in-place iteration
  - https://github.com/rust-lang/rust/pull/87667#issuecomment-1072607965
- yaahc/eyre: Add must-install feature, so that a non-default handler can be the on…
  - https://github.com/yaahc/eyre/pull/52#issuecomment-1074453007
  - https://github.com/yaahc/eyre/pull/52#issuecomment-1074465302
- rust-lang/rust: Mark `uint::wrapping_next_power_of_two` as `#[inline]`
  - https://github.com/rust-lang/rust/pull/94517#issuecomment-1072670466
  - https://github.com/rust-lang/rust/pull/94517#issuecomment-1072873639
  - https://github.com/rust-lang/rust/pull/94517#issuecomment-1074501841
- rust-lang/rust: Add `Option::inspect_none` & minor example improvements to other new `inspect` methods
  - https://github.com/rust-lang/rust/pull/94317#issuecomment-1072877582
  - https://github.com/rust-lang/rust/pull/94317#issuecomment-1074442258
- rust-lang/rust: Let `try_collect` take advantage of `try_fold` overrides
  - https://github.com/rust-lang/rust/pull/94115#issuecomment-1072665477
- rust-lang/rust: Upgrade libc to fix `Instant + Duration` producing wrong result on aarch64-apple-darwin
  - https://github.com/rust-lang/rust/pull/94100#issuecomment-1072632579
- rust-lang/project-error-handling: no-panic as a language feature
  - https://github.com/rust-lang/project-error-handling/issues/49#issuecomment-1073156185
- rust-lang/rust: Create `NonZero` trait with primitive associated type
  - https://github.com/rust-lang/rust/pull/95155#issuecomment-1074460757
- rust-lang/rust: Clean up, categorize and sort unstable features in std.
  - https://github.com/rust-lang/rust/pull/95032#issuecomment-1071197976
  - https://github.com/rust-lang/rust/pull/95032#issuecomment-1071198181
- yaahc/color-eyre: Allow omitting location/track-caller via function
  - https://github.com/yaahc/color-eyre/issues/105#issuecomment-1071017358
  - https://github.com/yaahc/color-eyre/issues/105#issuecomment-1071051305
- yaahc/eyre: Integrating with actix-web
  - https://github.com/yaahc/eyre/issues/72#issuecomment-1068446864
  - https://github.com/yaahc/eyre/issues/72#issuecomment-1068449557
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
