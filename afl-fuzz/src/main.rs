fn main() {
    afl::fuzz!(|data: &str| {
        let _: Result<rosu_map::Beatmap, _> = rosu_map::from_str(data);
    })
}
