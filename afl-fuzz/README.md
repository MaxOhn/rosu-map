## Fuzz tester

This is an AFL++-based fuzz tester that catches various unhandled panics within the library.

## How to test

```
cargo install cargo-afl
cargo afl build --release
# Optional to minimize corpus (be sure to update below command as well): cargo afl cmin -i ../resources -o ../resources_min -- target/release-afl-fuzz
cargo afl fuzz -i ../resources/ -o out/ target/release/afl-fuzz
```

## How to recreate a crash

```
cargo afl run --release target/release/afl-fuzz < out/default/crashes/[FILE_NAME]
```

## How to minimize a crash to find the minimum crashing input

```
cargo afl tmin -i out/default/crashes/[FILE_NAME] -o minimized_crash.osu target/release/afl-fuzz
```