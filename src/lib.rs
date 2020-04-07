#![warn(clippy::pedantic, clippy::nursery)]

//! `CharStream` is a hacked bidirectional char iterator that takes ownership of a
//! `String` and grants the client an ability to scan back and forth through a stream
//! of characters. A `CharStream` is not a ring; an attempt to iterate past the front or end
//! of the stream will fail and return None.
//!
//! This structure is designed to allow the caller to hold an immutable
//! instance to `CharStream`, since only the underlying implementation details of
//! CharStream need to change. This is done using interior mutability.
use std::cell::RefCell;

#[derive(Debug, PartialEq)]
pub struct CharStream {
    chars: Vec<char>,
    index: RefCell<isize>,
}

impl CharStream {
    /// Constructs a new CharStream from a String.
    pub fn from(s: String) -> Self {
        CharStream {
            chars: s.chars().collect(),
            index: RefCell::new(-1),
        }
    }
}

pub trait BiDirectionalIterator {
    fn next(&self) -> Option<char>;
    fn prev(&self) -> Option<char>;
    fn peek_next(&self) -> Option<&CharStream>;
    fn peek_prev(&self) -> Option<&CharStream>;
    fn value(&self) -> char;
}

impl BiDirectionalIterator for CharStream {
    /// Advance the `CharStream` by 1 returning the character
    fn next(&self) -> Option<char> {
        let current = *self.index.borrow() + 1;
        self.index.replace(current);

        if current >= self.chars.len() as isize {
            return None;
        }

        assert!(current >= 0);
        Some(self.chars[current as usize])
    }

    /// Retreat the `CharStream` by 1 returning the character
    fn prev(&self) -> Option<char> {
        let current = *self.index.borrow();
        if current == 0 {
            return None;
        }

        let current = *self.index.borrow() - 1;
        self.index.replace(current);

        assert!(current >= 0);
        Some(self.chars[current as usize])
    }

    /// Advance the CharStream by 1 returning &self
    fn peek_next(&self) -> Option<&CharStream> {
        let current = *self.index.borrow() + 1;
        self.index.replace(current);

        if current >= self.chars.len() as isize {
            return None;
        }

        Some(self)
    }

    /// Retreat the CharStream by 1 returning &self
    fn peek_prev(&self) -> Option<&CharStream> {
        let current = *self.index.borrow() - 1;
        if current < 0 {
            return None;
        }
        self.index.replace(current);
        Some(self)
    }

    /// Get the current value under the *cursor*
    fn value(&self) -> char {
        let current = *self.index.borrow();
        self.chars[current as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_get_the_next() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        assert_eq!(Some('f'), stream.next());
    }

    #[test]
    fn it_can_get_the_prev() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        stream.next(); // 'o'
        assert_eq!(Some('f'), stream.prev());
    }

    #[test]
    fn it_wont_step_off_the_front() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        assert_eq!(None, stream.prev());
    }

    #[test]
    fn it_wont_step_off_the_end() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        stream.next(); // 'o'
        stream.next(); // 'o'
        stream.next(); // 'b'
        stream.next(); // 'a'
        stream.next(); // 'r'
        assert_eq!(None, stream.next());
    }

    #[test]
    fn it_wont_step_off_the_front_peek() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        assert_eq!(None, stream.peek_prev());
    }

    #[test]
    fn it_wont_step_off_the_end_peek() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.peek_next(); // 'f'
        stream.peek_next(); // 'o'
        stream.peek_next(); // 'o'
        stream.peek_next(); // 'b'
        stream.peek_next(); // 'a'
        stream.peek_next(); // 'r'
        assert_eq!(None, stream.peek_next().map(CharStream::value));
    }

    #[test]
    fn it_can_get_the_peek_next() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        assert_eq!(Some('f'), stream.peek_next().map(CharStream::value));
    }

    #[test]
    fn it_can_get_the_peek_prev() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        assert!(stream.peek_next().is_some());
        assert_eq!(
            Some('f'),
            stream
                .peek_next()
                .and_then(CharStream::peek_prev)
                .map(CharStream::value)
        );
    }

    #[test]
    fn it_can_get_back_to_where_it_started() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        stream.next(); // 'o'
        stream.next(); // 'o'
        stream.next(); // 'b'
        stream.next(); // 'a'
        stream.next(); // 'r'
        stream.prev(); // 'a'
        stream.prev(); // 'b'
        stream.prev(); // 'o'
        stream.prev(); // 'o'
        assert_eq!(Some('f'), stream.prev());
    }

    #[test]
    fn it_can_get_back_to_where_it_started_peek() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.peek_next(); // 'f'
        stream.peek_next(); // 'o'
        stream.peek_next(); // 'o'
        stream.peek_next(); // 'b'
        stream.peek_next(); // 'a'
        stream.peek_next(); // 'r'
        stream.peek_prev(); // 'a'
        stream.peek_prev(); // 'b'
        stream.peek_prev(); // 'o'
        stream.peek_prev(); // 'o'
        assert_eq!(Some('f'), stream.prev());
    }
}
