pub fn key_bytes(key: &str) -> Option<Vec<u8>> {
    (key == "enter").then(|| vec![b'\r'])
}
