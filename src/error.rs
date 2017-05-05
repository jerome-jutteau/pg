use std::fmt;
use std::error;
use std::ptr;
use packetgraph_sys::{pg_error,
                      pg_error_is_set,
					  pg_error_free};

#[derive(Debug)]
pub struct Error {
    pub ptr: *mut pg_error,
    comment: String,
}

impl Error {
    pub fn new() -> Error {
        Error {
            ptr: ptr::null_mut(),
            comment: String::new(),
        }
    }

    pub fn set<S: Into<String>>(&mut self, comment: S) {
        self.comment = comment.into();
    }

    pub fn is_set(&mut self) -> bool {
        unsafe {
            return pg_error_is_set(&mut self.ptr);
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Packetgraph error: [TODO]")
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
		"todo"
    }
}

impl Drop for Error {
    fn drop(&mut self) {
        unsafe {
			pg_error_free(self.ptr);
		}
    }
}
