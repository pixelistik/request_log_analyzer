use io;
use std::fs::File;

pub struct MultiFile {
    files_iterator: Box<Iterator<Item = String>>,
    current_file: Option<File>,
}

impl MultiFile {
    pub fn new(filenames: Box<Iterator<Item = String>>) -> MultiFile {
        MultiFile {
            files_iterator: filenames,
            current_file: None,
        }
    }
}

impl io::Read for MultiFile {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let read_size;
        {
            let mut file = match self.current_file {
                Some(ref file) => file,
                None => {
                    println!("No file yet");
                    self.current_file = Some(File::open(&self.files_iterator.next().unwrap())?);
                    println!("Opened file {:#?}", &self.current_file);
                    return self.read(&mut buf);
                }
            };
            read_size = file.read(buf);
        }

        match read_size {
            Ok(0) => {
                println!("Read 0 bytes, current file {:#?}", &self.current_file);
                self.current_file = match &self.files_iterator.next() {
                    Some(file) => Some(File::open(file).unwrap()),
                    None => {
                        return Ok(0);
                    }
                };
                println!("Opened file {:#?}", &self.current_file);
                return self.read(&mut buf);
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
    fn test_instantiate() {
        let filenames = Box::new(
            vec![
                String::from("src/test/simple-1.log"),
                String::from("src/test/simple-2.log"),
            ].into_iter(),
        );
        let input = MultiFile::new(filenames);
    }

    #[test]
    fn test_read_all() {
        let filenames = Box::new(
            vec![
                String::from("src/test/simple-1.log"),
                String::from("src/test/simple-2.log"),
            ].into_iter(),
        );
        let mut input = MultiFile::new(filenames);

        let reader = io::BufReader::new(input);

        let result = reader.lines().count();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_read_non_existent() {
        let filenames = Box::new(vec![String::from("src/test/non-existent.log")].into_iter());
        let mut input = MultiFile::new(filenames);
        let mut buffer = [0; 10];
        let result = input.read(&mut buffer);

        assert!(result.is_err());
    }
}
