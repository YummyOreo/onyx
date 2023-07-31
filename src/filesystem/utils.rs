use std::path::PathBuf;

pub enum FileType {
    File(Option<&str>),
    Folder,
}

pub fn get_type_by_name(name: &str) -> FileType {
    if name.ends_with('\\') || name.ends_with('/') {
        FileType::Folder
    } else {
        PathBuf::from(name).extension().map(ToString::to_string)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn type_by_name_tests() {
        let paths = [
            ("test.txt", FileType::File(Some("txt"))),
            ("test", FileType::File(None)),
            ("test\\", FileType::Folder),
            ("test/", FileType::Folder),
        ];

        for (path, expected) in paths {
            assert_eq!(expected, get_type_by_name(path));
        }
    }
}
