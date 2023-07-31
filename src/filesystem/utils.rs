use std::path::Path;

const FILE_EXTENTION_REGEX: &str = r"\.([0-9a-zA-Z]+)$";

#[derive(PartialEq, Debug)]
pub enum FileType<'a> {
    File(Option<&'a str>),
    Folder,
}

pub fn get_type_by_name(name: &str) -> FileType {
    if name.ends_with('\\') || name.ends_with('/') {
        FileType::Folder
    } else {
        FileType::File(
            regex::Regex::new(FILE_EXTENTION_REGEX)
                .unwrap()
                .captures(name)
                .map(|c| c.get(1).unwrap().as_str()),
        )
    }
}

pub fn get_type_by_path(file: &Path) -> FileType {
    if file.is_dir() {
        FileType::Folder
    } else {
        FileType::File(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
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
