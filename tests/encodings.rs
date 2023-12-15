use rosu_map::beatmap::Beatmap;

#[test]
fn utf8_no_bom() {
    let bytes = b"osu file format v42\n\n";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);

    let bytes = b"\x20\x09\n\nosu file format v42\n\n";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);
}

#[test]
fn utf8_bom() {
    let bytes = b"\xEF\xBB\xBFosu file format v42\n\n";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);

    let bytes = b"\xEF\xBB\xBF\x20\x09\n\nosu file format v42\n\n";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);
}

#[test]
fn utf16_le() {
    let bytes = b"\xFF\xFEo\0s\0u\0 \0f\0i\0l\0e\0 \0f\0o\0r\0m\0a\0t\0 \0v\04\02\0\n\0\n\0";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);

    let bytes = b"\xFF\xFE\x09\x20\n\0\n\0o\0s\0u\0 \0f\0i\0l\0e\0 \0f\0o\0r\0m\0a\0t\0 \0v\04\02\0\n\0\n\0";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);
}

#[test]
fn utf16_be() {
    let bytes = b"\xFE\xFF\0o\0s\0u\0 \0f\0i\0l\0e\0 \0f\0o\0r\0m\0a\0t\0 \0v\04\02\0\n\0\n";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);

    let bytes = b"\xFE\xFF\x20\x09\0\n\0\n\0o\0s\0u\0 \0f\0i\0l\0e\0 \0f\0o\0r\0m\0a\0t\0 \0v\04\02\0\n\0\n";
    let map = Beatmap::from_bytes(bytes).unwrap();
    assert_eq!(map.format_version, 42);
}
