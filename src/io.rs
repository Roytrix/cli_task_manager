// Module: io
use std::any::Any;
use std::io;

pub trait IO {
    fn read_line(&mut self) -> io::Result<String>;
    fn write_line(&mut self, line: &str) -> io::Result<()>;
    fn as_any(&self) -> &dyn Any;
}

pub struct ConsoleIO;

impl IO for ConsoleIO {
    fn read_line(&mut self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn write_line(&mut self, line: &str) -> io::Result<()> {
        println!("{}", line);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests_io {
    use super::*;
    use std::io::{self};

    struct MockIO {
        input: Vec<String>,
        output: Vec<String>,
    }

    impl MockIO {
        fn new(input: Vec<String>) -> Self {
            Self {
                input,
                output: Vec::new(),
            }
        }
    }

    impl IO for MockIO {
        fn read_line(&mut self) -> io::Result<String> {
            if let Some(line) = self.input.pop() {
                Ok(line)
            } else {
                Ok(String::new())
            }
        }

        fn write_line(&mut self, line: &str) -> io::Result<()> {
            self.output.push(line.to_string());
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn read_line_returns_correct_string() {
        let mut mock_io = MockIO::new(vec!["Hello, world!".to_string()]);
        let result = mock_io.read_line().unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn read_line_handles_empty_input() {
        let mut mock_io = MockIO::new(vec!["".to_string()]);
        let result = mock_io.read_line().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn read_line_returns_empty_string_when_no_input() {
        let mut mock_io = MockIO::new(vec![]);
        let result = mock_io.read_line().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn read_line_handles_multiple_inputs() {
        let mut mock_io = MockIO::new(vec!["First".to_string(), "Second".to_string()]);
        let result1 = mock_io.read_line().unwrap();
        let result2 = mock_io.read_line().unwrap();
        assert_eq!(result1, "Second");
        assert_eq!(result2, "First");
    }

    #[test]
    fn write_line_outputs_correct_string() {
        let mut mock_io = MockIO::new(vec![]);
        let line = "Hello, world!";
        mock_io.write_line(line).unwrap();
        assert_eq!(mock_io.output, vec![line.to_string()]);
    }

    #[test]
    fn write_line_handles_empty_string() {
        let mut mock_io = MockIO::new(vec![]);
        let line = "";
        mock_io.write_line(line).unwrap();
        assert_eq!(mock_io.output, vec![line.to_string()]);
    }
}
