use std::{
    fmt::{
        Debug,
        Display,
    },
    thread::{
        self,
        ThreadId,
    },
};

pub struct ThreadLocalCell<T> {
    inner: T,
    created_on: ThreadId,
}

impl<T> ThreadLocalCell<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            created_on: thread::current().id(),
        }
    }

    pub fn get(&self) -> &T {
        self.try_get().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.try_get_mut().unwrap()
    }

    pub fn try_get(&self) -> Result<&T, ThreadLocalCellError> {
        check_thread(self.created_on)?;
        Ok(&self.inner)
    }

    pub fn try_get_mut(&mut self) -> Result<&mut T, ThreadLocalCellError> {
        check_thread(self.created_on)?;
        Ok(&mut self.inner)
    }
}

impl<T: Debug> Debug for ThreadLocalCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("ThreadLocalCell");
        debug_struct.field("created_on", &self.created_on);
        if let Ok(inner) = self.try_get() {
            debug_struct.field("inner", inner).finish()
        }
        else {
            debug_struct.finish_non_exhaustive()
        }
    }
}

impl<T> Drop for ThreadLocalCell<T> {
    fn drop(&mut self) {
        check_thread(self.created_on).unwrap();
    }
}

unsafe impl<T> Send for ThreadLocalCell<T> {}
unsafe impl<T> Sync for ThreadLocalCell<T> {}

fn check_thread(created_on: ThreadId) -> Result<(), ThreadLocalCellError> {
    let accessed_on = std::thread::current().id();
    if created_on == accessed_on {
        Ok(())
    }
    else {
        Err(ThreadLocalCellError {
            created_on,
            accessed_on,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Tried to access a ThreadLocalCell from a different thread")]
pub struct ThreadLocalCellError {
    pub created_on: ThreadId,
    pub accessed_on: ThreadId,
}

#[derive(Debug)]
pub struct ThreadLocalError<E> {
    message: String,
    error: ThreadLocalCell<E>,
}

impl<E: Display> ThreadLocalError<E> {
    pub fn new(error: E) -> Self {
        Self {
            message: error.to_string(),
            error: ThreadLocalCell::new(error),
        }
    }

    pub fn try_get(&self) -> Result<&E, ThreadLocalCellError> {
        self.error.try_get()
    }
}

impl<E> Display for ThreadLocalError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<E: std::error::Error> std::error::Error for ThreadLocalError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.try_get().ok()?.source()
    }
}
