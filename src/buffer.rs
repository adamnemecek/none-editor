use ropey;
use ropey::Rope;
use std::fs::File;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// A text Buffer
#[derive(Debug, Clone)]
pub struct Buffer {
    rope: Rope,
    filename: Option<PathBuf>,
    is_dirty: bool,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Buffer {
            rope: Rope::new(),
            filename: None,
            is_dirty: false,
        }
    }
    /// create a buffer from the given string
    pub fn from_str(text: &str) -> Self {
        Buffer {
            rope: Rope::from_str(text),
            filename: None,
            is_dirty: false,
        }
    }
    /// create a buffer from the give file
    pub fn from_file(filename: &Path) -> Result<Self, io::Error> {
        let r = Rope::from_reader(io::BufReader::new(File::open(filename)?))?;
        Ok(Buffer {
            rope: r,
            filename: Some(filename.to_owned()),
            is_dirty: false,
        })
    }

    /// return the filename
    pub fn get_filename<'a>(&'a self) -> Option<&'a Path> {
        match &self.filename {
            Some(p) => Some(p.as_path()),
            None => None
        }
    }

    /// Iterate over each char in the buffer
    pub fn chars(&self) -> ropey::iter::Chars {
        self.rope.chars()
    }
    pub fn lines(&self) -> ropey::iter::Lines {
        self.rope.lines()
    }
    /// Total number of chars in the buffer
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }
    /// Total number of lines in the buffer
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }
    /// insert ch at the given position
    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        self.rope.insert_char(char_idx, ch);
        self.is_dirty = true;
    }
    /// Insert the string at the given position
    pub fn insert<S: AsRef<str>>(&mut self, char_idx: usize, text: S) {
        self.rope.insert(char_idx, text.as_ref());
        self.is_dirty = true;
    }
    /// remove the given range from the buffer
    pub fn remove(&mut self, char_range: Range<usize>) {
        self.rope.remove(char_range);
        self.is_dirty = true;
    }

    /// Returns the entire buffer as a newly allocated String.
    pub fn to_string(&self) -> String {
        self.rope.to_string()
    }
    pub fn slice(&self,r: Range<usize>) -> String {
        self.rope.slice(r).to_string()
    }

    /// return the line of the given char
    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }
    /// return the first char of the given line
    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    pub fn line_len_no_eol(&self, line_idx: usize) -> usize {
        let l = self.rope.line(line_idx);
        l.chars().filter(|c| *c!='\n' && *c!='\r').count()
    }

    /// return the last char of the given line
    pub fn line_to_last_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx) + self.line_len_no_eol(line_idx)
    }

    /// return the len in chars of the given line
    pub fn line_len(&self, line_idx: usize) -> usize {
        self.rope.line(line_idx).len_chars()
    }
    /// convert an index to a point (line, column)
    pub fn index_to_point(&self, char_idx: usize) -> (usize, usize) {
        let l = self.char_to_line(char_idx);
        let c = char_idx - self.line_to_char(l);
        (l, c)
    }
    /// Convert a point (line, column) to an index
    pub fn point_to_index(&self, point: (usize, usize)) -> usize {
        use std::cmp::min;
        let l = min(point.0, self.len_lines() - 1);

        let c = min(point.1, self.line_len_no_eol(l) );
        self.line_to_char(l) + c
    }
}

#[cfg(test)]
mod tests {
    use buffer::Buffer;

    #[test]
    fn chars_iterators() {
        let buf = Buffer::from_str("Hello World");
        let res = ['H', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd'];
        let mut i = 0;
        for c in buf.chars() {
            assert_eq!(c, res[i]);
            i += 1;
        }
    }

    #[test]
    fn len_chars() {
        let buf = Buffer::from_str("Hello World");
        assert_eq!(buf.len_chars(), 11);
        let buf = Buffer::from_str("Hello World\n");
        assert_eq!(buf.len_chars(), 12);
        let buf = Buffer::from_str("Nöel");
        assert_eq!(buf.len_chars(), 4);
    }
    #[test]
    fn len_lines() {
        let buf = Buffer::from_str("Hello World");
        assert_eq!(buf.len_lines(), 1);
        let buf = Buffer::from_str("Hello\nWorld");
        assert_eq!(buf.len_lines(), 2);
    }
    #[test]
    fn remove() {
        let mut buf = Buffer::from_str("Hello World");
        buf.remove(1..3);
        assert_eq!(buf.to_string(), "Hlo World");
    }
    #[test]
    fn index_to_point() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        assert_eq!(buf.index_to_point(3), (0, 3));
        assert_eq!(buf.index_to_point(4), (0, 4));
        assert_eq!(buf.index_to_point(5), (1, 0));
        assert_eq!(buf.index_to_point(12), (2, 1));
    }
    #[test]
    fn point_to_index() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        // Normal case
        assert_eq!(buf.point_to_index((0, 3)), 3);
        assert_eq!(buf.point_to_index((0, 4)), 4);
        assert_eq!(buf.point_to_index((1, 0)), 5);
        assert_eq!(buf.point_to_index((2, 1)), 12);

        // oob case
        assert_eq!(buf.point_to_index((0, 5)), 4); // col too far
        assert_eq!(buf.point_to_index((4, 1)), 12); // line too far
        assert_eq!(buf.point_to_index((4, 6)), 17); // line too far, EOF is treated like a char
    }
    #[test]
    fn line_to_last_char() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        assert_eq!(buf.line_to_last_char(0), 4);
        assert_eq!(buf.line_to_last_char(1), 10);
        assert_eq!(buf.line_to_last_char(2), 17); // EOF is treated like à char
    }
    #[test]
    fn line_len_no_eol() {
        let buf = Buffer::from_str("text\nplops\ntoto  ");
        assert_eq!(buf.line_len_no_eol(0), 4);
        assert_eq!(buf.line_len_no_eol(1), 5);
        assert_eq!(buf.line_len_no_eol(2), 6);
    }
}
