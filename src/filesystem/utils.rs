use std::path::PathBuf;

const FILEEXTENTION_MATCH: &str = &r"\.([0-9a-zA-Z]+)$";

#[derive(PartialEq, Debug)]
pub enum FileType<'a> {
    File(Option<&'a str>),
    Folder,
}

pub fn get_type_by_name<'a>(name: &'a str) -> FileType<'a> {
    if name.ends_with('\\') || name.ends_with('/') {
        FileType::Folder
    } else {
        FileType::File(
            regex::Regex::new(FILEEXTENTION_MATCH)
                .unwrap()
                .captures(name)
                .map(|c| c.get(1).unwrap().as_str()),
        )
    }
}

pub fn get_type<'a>(file: &'a PathBuf) -> FileType<'a> {
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
