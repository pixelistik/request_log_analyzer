use std::io;
use std::fs::File;
use flate2::read::GzDecoder;

pub struct MultiFile {
    files_iterator: Box<dyn Iterator<Item = String>>,
    current_file: Option<Box<dyn io::Read>>,
}

impl MultiFile {
    pub fn new(filenames: Vec<String>) -> MultiFile {
        MultiFile {
            files_iterator: Box::new(filenames.into_iter()),
            current_file: None,
        }
    }
}

impl io::Read for MultiFile {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let read_size = match self.current_file {
            Some(ref mut file) => file.read(buf),
            None => {
                self.current_file = match &self.files_iterator.next() {
                    Some(file) if file.ends_with(".gz") => Some(Box::new(
                        GzDecoder::new(File::open(file)?),
                    )),
                    Some(file) => Some(Box::new(File::open(file)?)),
                    None => {
                        return Ok(0);
                    }
                };
                return self.read(&mut buf);
            }
        };

        match read_size {
            Ok(0) => {
                // EOF, proceed with next file
                self.current_file = None;
                self.read(&mut buf)
            }
            Ok(read_size) => Ok(read_size),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Read;
    use std::io::BufRead;

    use super::*;

    #[test]
    fn test_read_single() {
        let filenames = vec![String::from("src/test/simple-1.log")];
        let input = MultiFile::new(filenames);

        let reader = io::BufReader::new(input);

        let result = reader.lines().count();
        assert_eq!(result, 4);
    }

    #[test]
    fn test_read_all() {
        let filenames = vec![
            String::from("src/test/simple-1.log"),
            String::from("src/test/simple-2.log"),
        ];
        let input = MultiFile::new(filenames);

        let reader = io::BufReader::new(input);

        let result = reader.lines().count();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_read_non_existent() {
        let filenames = vec![String::from("src/test/non-existent.log")];
        let mut input = MultiFile::new(filenames);
        let mut buffer = [0; 10];
        let result = input.read(&mut buffer);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_second_non_existent() {
        let filenames = vec![
            String::from("src/test/empty.log"),
            String::from("src/test/non-existent.log"),
        ];
        let mut input = MultiFile::new(filenames);
        let mut buffer = [0; 10];
        let result = input.read(&mut buffer);

        assert!(result.is_err());
    }

    #[test]
    fn test_read_no_files_means_eof() {
        let filenames = vec![];
        let mut input = MultiFile::new(filenames);
        let mut buffer = [0; 10];
        let result = input.read(&mut buffer);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_read_with_gzip() {
        let filenames = vec![
            String::from("src/test/simple-1.log"),
            String::from("src/test/simple-1.log.gz"),
        ];
        let input = MultiFile::new(filenames);

        let reader = io::BufReader::new(input);

        let result = reader.lines().count();
        assert_eq!(result, 8);
    }
}
