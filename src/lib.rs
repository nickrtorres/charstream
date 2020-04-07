#![warn(clippy::pedantic, clippy::nursery)]

//! CharStream is a hacked bi-directional char iterator that takes ownership of a
//! `String` and grants the client an ability to scan back and forth through this
//! string. A CharStream is not a ring; an attempt to iterate past the front or end
//! of the stream will fail with CharStreamError::FallsOffEnd
//!
//! CharStream is takes 2N in space where N is the number of characters in the
//! original string. This is guaranteed as a Vector holding an internal cache of
//! the String is allocated on construction. This not ideal.
//!
//! This value structure is designed to allow the caller to hold an immutable
//! instance to CharStream, since only the underlying implementation details of
//! CharStream need to change. This is done using interior mutability.
use std::cell::RefCell;

#[derive(Debug, PartialEq)]
pub enum CharStreamError {
    /// An attempt was made to walk off either end of the CharStream.
    NextFailed,
    /// A call to String::chars().next() failed. This is fatal as the internal
    /// structure of the CharStream is now malformed.
    FallsOffEnd,
    /// CharStream's internal buffer was unwrapped to None. This is a
    /// programming error
    ValueNotFound,
}

#[derive(Debug, PartialEq)]
pub struct CharStream {
    payload: RefCell<Vec<Option<char>>>,
    value: String,
    index: RefCell<usize>,
}

impl CharStream {
    /// Constructs a new CharStream from a String.
    ///
    /// Allocate a vector with a len() 1 greater than the s.len(). This is so
    /// that we can use index 0 as a sentinal value.
    pub fn from(s: String) -> Self {
        CharStream {
            payload: RefCell::new(vec![None; s.len() + 1]),
            value: s,
            index: RefCell::new(0),
        }
    }
}

pub trait BiDirectionalIterator {
    fn next(&self) -> Result<char, CharStreamError>;
    fn prev(&self) -> Result<char, CharStreamError>;
    fn peek_next(&self) -> Result<&CharStream, CharStreamError>;
    fn peek_prev(&self) -> Result<&CharStream, CharStreamError>;
    fn value(&self) -> Result<char, CharStreamError>;
}

impl BiDirectionalIterator for CharStream {
    /// Advance the CharStream by 1 returning the character
    ///
    /// # Errors
    /// CharStreamError::FallsOffEnd if a complete call to next would step off
    /// the end of the String
    ///
    /// CharStreamError::ValueNotFound if indexing into a *good* index is None.
    /// This is a programming error.
    fn next(&self) -> Result<char, CharStreamError> {
        let current = *self.index.borrow() + 1;
        self.index.replace(current);

        if current > self.value.len() {
            return Err(CharStreamError::FallsOffEnd);
        }

        // we've already been here. early return
        if self.payload.borrow()[current].is_some() {
            return self.payload.borrow()[current].ok_or(CharStreamError::ValueNotFound);
        }

        if let Some(c) = self.value.chars().next() {
            self.payload.borrow_mut()[current] = Some(c);
            self.payload.borrow()[current].ok_or(CharStreamError::ValueNotFound)
        } else {
            return Err(CharStreamError::FallsOffEnd);
        }
    }

    /// Retreat the CharStream by 1 returning the character
    ///
    /// # Errors
    /// CharStreamError::FallsOffEnd if a complete call to prev would step off
    /// the beginning of the String
    ///
    /// CharStreamError::ValueNotFound if indexing into a *good* index is None.
    /// This is a programming error.
    fn prev(&self) -> Result<char, CharStreamError> {
        let current = *self.index.borrow();
        if current == 1 {
            return Err(CharStreamError::FallsOffEnd);
        }

        let current = *self.index.borrow() - 1;
        self.index.replace(current);

        let val = self.payload.borrow()[current];
        assert!(current == 0 || self.payload.borrow()[current].is_some());
        val.ok_or(CharStreamError::ValueNotFound)
    }

    /// Advance the CharStream by 1 returning &self
    ///
    /// # Errors
    /// CharStreamError::FallsOffEnd if a complete call to prev would step off
    /// the end of the String
    ///
    /// CharStreamError::ValueNotFound if indexing into a *good* index is None.
    /// This is a programming error.
    ///
    /// CharStreamError::NextFailed if calling next on the internal String
    /// fails. This error is fatal.
    fn peek_next(&self) -> Result<&CharStream, CharStreamError> {
        let current = *self.index.borrow() + 1;
        self.index.replace(current);

        if current > self.value.len() {
            return Err(CharStreamError::FallsOffEnd);
        }

        // we've already been here. early return
        if self.payload.borrow()[current].is_some() {
            return Ok(self);
        } else if let Some(c) = self.value.chars().next() {
            self.payload.borrow_mut()[current] = Some(c);
            assert!(self.payload.borrow()[current].is_some());
        } else {
            return Err(CharStreamError::NextFailed);
        }

        Ok(self)
    }

    /// Retreat the CharStream by 1 returning &self
    ///
    /// # Errors
    /// CharStreamError::FallsOffEnd if a complete call to prev would step off
    /// the beginning of the String
    ///
    /// CharStreamError::ValueNotFound if indexing into a *good* index is None.
    /// This is a programming error.
    ///
    /// CharStreamError::NextFailed if calling next on the internal String
    /// fails. This error is fatal.
    fn peek_prev(&self) -> Result<&CharStream, CharStreamError> {
        let current = *self.index.borrow() - 1;
        if current == 0 {
            return Err(CharStreamError::FallsOffEnd);
        }
        self.index.replace(current);

        assert!(current == 0 || self.payload.borrow()[current].is_some());
        Ok(self)
    }

    /// Get the current value under the *cursor*
    ///
    /// # Errors
    /// CharStreamError::ValueNotFound if indexing into an unitialized
    /// CharStream (i.e. next() hasn't been called)
    /// fails. This error is fatal.
    fn value(&self) -> Result<char, CharStreamError> {
        let current = *self.index.borrow();
        self.payload.borrow()[current].ok_or(CharStreamError::ValueNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_get_the_next() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        assert_eq!(Ok('f'), stream.next());
    }

    #[test]
    fn it_can_get_the_prev() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        stream.next(); // 'o'
        assert_eq!(Ok('f'), stream.prev());
    }

    #[test]
    fn it_wont_step_off_the_front() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        assert_eq!(Err(CharStreamError::FallsOffEnd), stream.prev());
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
        assert_eq!(Err(CharStreamError::FallsOffEnd), stream.next());
    }

    #[test]
    fn it_wont_step_off_the_front_peek() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        stream.next(); // 'f'
        assert_eq!(Err(CharStreamError::FallsOffEnd), stream.peek_prev());
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
        assert_eq!(
            Err(CharStreamError::FallsOffEnd),
            stream.peek_next().and_then(CharStream::value)
        );
    }

    #[test]
    fn it_can_get_the_peek_next() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        assert_eq!(Ok('f'), stream.peek_next().and_then(CharStream::value));
    }

    #[test]
    fn it_can_get_the_peek_prev() {
        let value = String::from("foobar");
        let stream = CharStream::from(value);
        assert!(stream.peek_next().is_ok());
        assert_eq!(
            Ok('f'),
            stream
                .peek_next()
                .and_then(CharStream::peek_prev)
                .and_then(CharStream::value)
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
        assert_eq!(Ok('f'), stream.prev());
    }

    #[test]
    fn it_can_get_back_to_where_it_started_peek() {
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
        assert_eq!(Ok('f'), stream.prev());
    }
}
