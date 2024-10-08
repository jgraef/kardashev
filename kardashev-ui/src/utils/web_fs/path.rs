use std::{
    borrow::Borrow,
    convert::Infallible,
    fmt::{
        Debug,
        Display,
    },
    iter::FusedIterator,
    ops::{
        Deref,
        DerefMut,
    },
    str::{
        CharIndices,
        FromStr,
    },
};

use serde::{
    Deserialize,
    Serialize,
};

pub const SEPARATOR: char = '/';
pub const SEPARATOR_STR: &'static str = "/";

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Path {
    inner: str,
}

impl Path {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Self {
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    fn from_inner_mut(s: &mut str) -> &mut Self {
        unsafe { &mut *(s as *mut str as *mut Path) }
    }

    pub fn is_absolute(&self) -> bool {
        self.inner
            .chars()
            .next()
            .map_or(false, |first| first == SEPARATOR)
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    pub fn components(&self) -> Components {
        Components::new(self)
    }

    pub fn ends_with(&self, child: impl AsRef<Path>) -> bool {
        let mut self_iter = self.components();
        let mut child_iter = child.as_ref().components();

        loop {
            match (self_iter.next_back(), child_iter.next_back()) {
                (Some(left), Some(right)) if left == right => {}
                (Some(_), None) => break,
                _ => return false,
            }
        }

        true
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for &'de Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        Ok(Path::new(s))
    }
}

impl ToOwned for Path {
    type Owned = PathBuf;

    fn to_owned(&self) -> Self::Owned {
        PathBuf {
            inner: self.inner.to_owned(),
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathBuf {
    inner: String,
}

impl PathBuf {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: String::with_capacity(capacity),
        }
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.inner)
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn push(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if path.is_absolute() {
            self.inner = path.inner.to_owned();
        }
        else {
            if self
                .inner
                .chars()
                .next_back()
                .map_or(false, |last| last == SEPARATOR)
            {
                self.inner.push(SEPARATOR);
            }
            self.inner.push_str(&path.inner);
        }
    }
}

impl Deref for PathBuf {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        Path::new(&self.inner)
    }
}

impl DerefMut for PathBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Path::from_inner_mut(&mut self.inner)
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl AsRef<str> for PathBuf {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl From<String> for PathBuf {
    fn from(value: String) -> Self {
        Self { inner: value }
    }
}

impl From<PathBuf> for String {
    fn from(value: PathBuf) -> Self {
        value.inner
    }
}

impl FromStr for PathBuf {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: s.to_owned(),
        })
    }
}

impl Borrow<Path> for PathBuf {
    fn borrow(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl Display for PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Debug for PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl Serialize for PathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PathBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Ok(Self { inner: s })
    }
}

#[derive(Clone, Debug)]
pub struct Components<'a> {
    path: &'a str,
    components: Split<'a>,
    forward_is_first: bool,
    back_previous_was_empty: bool,
    back_previous_was_curdir: bool,
}

impl<'a> Components<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path: &path.inner,
            components: Split::new(&path.inner, SEPARATOR),
            forward_is_first: true,
            back_previous_was_curdir: false,
            back_previous_was_empty: false,
        }
    }

    pub fn as_path(&self) -> &Path {
        self.remaining_path()
    }

    pub fn remaining_path(&self) -> &Path {
        todo!();
    }

    pub fn consumed_path(&self) -> &Path {
        todo!();
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let component = self.components.next()?;

            match component {
                "" => {
                    if self.forward_is_first {
                        self.forward_is_first = false;
                        return Some(Component::RootDir);
                    }
                }
                "." => {
                    if self.forward_is_first {
                        self.forward_is_first = false;
                        return Some(Component::CurDir);
                    }
                }
                ".." => {
                    self.forward_is_first = false;
                    return Some(Component::ParentDir);
                }
                _ => {
                    self.forward_is_first = false;
                    return Some(Component::Normal(component));
                }
            }
        }
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let Some(component) = self.components.next()
            else {
                if self.forward_is_first {
                    if self.back_previous_was_empty {
                        return Some(Component::RootDir);
                    }
                    else if self.back_previous_was_curdir {
                        return Some(Component::CurDir);
                    }
                }
                return None;
            };

            self.back_previous_was_empty = false;
            self.back_previous_was_curdir = false;

            match component {
                "" => {
                    self.back_previous_was_empty = true;
                }
                "." => {
                    self.back_previous_was_curdir = true;
                }
                ".." => {
                    return Some(Component::ParentDir);
                }
                _ => {
                    return Some(Component::Normal(component));
                }
            }
        }
    }
}

impl<'a> FusedIterator for Components<'a> {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Component<'a> {
    RootDir,
    CurDir,
    ParentDir,
    Normal(&'a str),
}

#[derive(Clone, Debug)]
struct Split<'a> {
    input: &'a str,
    char_indices: CharIndices<'a>,
    separator: char,
    pos_front: usize,
    pos_back: usize,
    done: bool,
}

impl<'a> Split<'a> {
    pub fn new(input: &'a str, separator: char) -> Self {
        Self {
            input,
            char_indices: input.char_indices(),
            separator,
            pos_front: 0,
            pos_back: input.len(),
            done: false,
        }
    }
}

impl<'a> Iterator for Split<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((start, mut c)) = self.char_indices.next() {
            let mut end = start;

            while c != self.separator {
                if let Some((pos, next_c)) = self.char_indices.next() {
                    c = next_c;
                    self.pos_front = pos + c.len_utf8();
                    end = pos;
                }
                else {
                    end = self.pos_back;
                    self.done = true;
                    break;
                }
            }

            Some(&self.input[start..end])
        }
        else if self.done {
            None
        }
        else {
            self.done = true;
            Some("")
        }
    }
}

impl<'a> DoubleEndedIterator for Split<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some((end, mut c)) = self.char_indices.next_back() {
            let end = end + c.len_utf8();
            let mut start = end;

            while c != self.separator {
                if let Some((pos, next_c)) = self.char_indices.next_back() {
                    c = next_c;
                    self.pos_back = pos;
                    start = pos + c.len_utf8();
                }
                else {
                    start = self.pos_front;
                    self.done = true;
                    break;
                }
            }

            Some(&self.input[start..end])
        }
        else if self.done {
            None
        }
        else {
            self.done = true;
            Some("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_splits_strs_forward() {
        fn split<'a>(s: &'a str) -> Vec<&'a str> {
            Split::new(s, '/').collect::<Vec<_>>()
        }

        assert_eq!(split("a/b/c"), vec!["a", "b", "c"]);
        assert_eq!(split("ab/cd/ef"), vec!["ab", "cd", "ef"]);
        assert_eq!(split("/a/b/c"), vec!["", "a", "b", "c"]);
        assert_eq!(split("a/b/c/"), vec!["a", "b", "c", ""]);
        assert_eq!(split("a//b/c"), vec!["a", "", "b", "c"]);
        assert_eq!(split("//a/b/c"), vec!["", "", "a", "b", "c"]);
        assert_eq!(split("a/b/c//"), vec!["a", "b", "c", "", ""]);
    }

    #[test]
    fn it_splits_strs_backwards() {
        fn split<'a>(s: &'a str) -> Vec<&'a str> {
            Split::new(s, '/').rev().collect::<Vec<_>>()
        }

        assert_eq!(split("a/b/c"), vec!["c", "b", "a"]);
        assert_eq!(split("ab/cd/ef"), vec!["ef", "cd", "ab"]);
        assert_eq!(split("/a/b/c"), vec!["c", "b", "a", ""]);
        assert_eq!(split("a/b/c/"), vec!["", "c", "b", "a"]);
        assert_eq!(split("a//b/c"), vec!["c", "b", "", "a"]);
        assert_eq!(split("//a/b/c"), vec!["c", "b", "a", "", ""]);
        assert_eq!(split("a/b/c//"), vec!["", "", "c", "b", "a"]);
    }

    #[test]
    fn it_splits_strs_both_ways() {
        let mut it = Split::new("a/b/c", '/');
        assert_eq!(it.next(), Some("a"));
        assert_eq!(it.next_back(), Some("c"));
        assert_eq!(it.next(), Some("b"));
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);

        let mut it = Split::new("a/b/c", '/');
        assert_eq!(it.next_back(), Some("c"));
        assert_eq!(it.next(), Some("a"));
        assert_eq!(it.next_back(), Some("b"));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn it_splits_paths_forwards() {
        fn split<'a>(path: &'a str) -> Vec<Component<'a>> {
            Path::new(path).components().collect::<Vec<_>>()
        }

        assert_eq!(
            split("/usr/bin"),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("/usr/./bin"),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("/usr/bin/."),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("/usr//bin"),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("//usr/bin"),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("/usr/bin/"),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("/usr/bin//"),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("./usr/bin"),
            vec![
                Component::CurDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("././usr/bin"),
            vec![
                Component::CurDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("usr/bin"),
            vec![Component::Normal("usr"), Component::Normal("bin")]
        );
        assert_eq!(
            split("/usr/../bin"),
            vec![
                Component::RootDir,
                Component::Normal("usr"),
                Component::ParentDir,
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("../usr/bin"),
            vec![
                Component::ParentDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split("./../usr/bin"),
            vec![
                Component::CurDir,
                Component::ParentDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
        assert_eq!(
            split(".././usr/bin"),
            vec![
                Component::ParentDir,
                Component::Normal("usr"),
                Component::Normal("bin")
            ]
        );
    }
}
