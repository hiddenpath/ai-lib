#[cfg(test)]
mod tests {
    use ai_lib::utils::file as file_utils;
    use std::path::Path;

    #[test]
    fn save_read_remove_roundtrip() {
        let data = b"hello-multimodal";
        let path = file_utils::save_temp_file("ai-lib-test", data).expect("save_temp_file");
        assert!(path.exists());

        let read = file_utils::read_file(&path).expect("read_file");
        assert_eq!(read.as_slice(), data);

    let mime = file_utils::guess_mime_from_path(&path);
    assert!(!mime.is_empty());

        file_utils::remove_file(&path).expect("remove_file");
        assert!(!Path::new(&path).exists());
    }
}
