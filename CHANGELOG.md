# v0.2.1 (2025-04-07)

- Fixed clamping for effect point scroll speed
- Fixed slider events with `0.0` tick distance

## v0.2.0 (2024-11-18)

- [Breaking] Creating a `Curve` or `BorrowedCurve`, and thus `SliderPath` too, now requires a specified `GameMode`. ([#5])
- The default implementation of the function `Decode::should_skip_line` has been slightly adjusted ([#5])

## v0.1.2 (2024-08-17)

Fixes niche panic on malicious .osu files ([#4])

## v0.1.1 (2024-03-11)

Removed the `thiserror` dependency, leading to potentially shorter compile times and smaller binaries. ([#1])

## v0.1.0 (2024-02-27)

First release

[#1]: https://github.com/MaxOhn/rosu-map/pull/1
[#4]: https://github.com/MaxOhn/rosu-map/pull/4
[#5]: https://github.com/MaxOhn/rosu-map/pull/5