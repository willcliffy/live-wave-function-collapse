use std::{
    cmp::min,
    sync::{Mutex, MutexGuard},
};

use chrono::Utc;
use godot::builtin::Vector3i;

pub trait Book {
    fn location(&self) -> Vector3i;

    fn version(&self) -> String;
    fn set_version(&mut self, version: String);

    fn is_checked_out(&self) -> bool;
    fn check_out(&mut self) -> bool;
    fn check_in(&mut self) -> bool;
}

pub struct Range<T> {
    pub size: Vector3i,
    pub books: Vec<T>,
}

impl<T> Range<T> {
    pub fn new(size: Vector3i, books: Vec<T>) -> Self {
        Self { size, books }
    }

    pub fn index(&self, location: Vector3i, start_offset: Vector3i) -> usize {
        ((location.y - start_offset.y) * (self.size.x * self.size.z)
            + (location.x - start_offset.x) * self.size.z
            + (location.z - start_offset.z)) as usize
    }
}

pub struct Library3D<T> {
    pub size: Vector3i,
    books: Mutex<Vec<T>>,
}

impl<T: Book + Clone> Library3D<T> {
    pub fn new(size: Vector3i, books: Vec<T>) -> Self {
        Self {
            size,
            books: Mutex::new(books),
        }
    }

    pub fn check_out_range(&self, start: Vector3i, end: Vector3i) -> anyhow::Result<Range<T>> {
        let mut books;
        match self.books.lock() {
            Ok(books_locked) => books = books_locked,
            Err(e) => return Err(anyhow::anyhow!("Failed to lock books: {}", e)),
        }

        let mut range_end = end.clone();
        range_end.x = min(range_end.x, self.size.x);
        range_end.y = min(range_end.y, self.size.y);
        range_end.z = min(range_end.z, self.size.z);

        let selection = self.get_selection(&books, start, range_end)?;

        let mut range_books = vec![];
        for i in selection {
            let book = books.get_mut(i).unwrap();
            book.check_out();
            range_books.push(book.clone())
        }

        Ok(Range::new(range_end - start, range_books))
    }

    pub fn check_in_range(&self, range: &mut Range<T>) -> anyhow::Result<()> {
        let mut books;
        match self.books.lock() {
            Ok(books_locked) => books = books_locked,
            Err(e) => return Err(anyhow::anyhow!("Failed to lock books: {}", e)),
        }

        for book in range.books.iter_mut() {
            let index = self.get_index(book.location());
            match books.get(index) {
                Some(current_book) => {
                    if current_book.version() != book.version() {
                        return Err(anyhow::anyhow!("Version mismatch on check-in: tried to check in from version {} but newest version is {}. book: {:?}", book.version(), current_book.version(), book.location()));
                    }
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "Invalid index on check-in: index {} (for position {}) is out of bounds {}",
                        index,
                        book.location(),
                        books.len()
                    ))
                }
            }
        }

        for book in range.books.iter_mut() {
            let index = self.get_index(book.location());
            book.set_version(Self::now());
            book.check_in();
            books[index] = book.clone();
        }

        Ok(())
    }

    pub fn copy_range(&self, start: Vector3i, end: Vector3i) -> anyhow::Result<Range<T>> {
        let books;
        match self.books.lock() {
            Ok(books_locked) => books = books_locked,
            Err(e) => return Err(anyhow::anyhow!("Failed to lock books: {}", e)),
        }

        let mut range_end = end.clone();
        range_end.x = min(range_end.x, self.size.x);
        range_end.y = min(range_end.y, self.size.y);
        range_end.z = min(range_end.z, self.size.z);

        let selection = self.get_selection(&books, start, range_end)?;

        let mut range_books = vec![];
        for i in selection {
            let book = books.get(i).unwrap();
            range_books.push(book.clone())
        }

        Ok(Range::new(range_end - start, range_books))
    }

    fn get_selection(
        &self,
        books: &MutexGuard<'_, Vec<T>>,
        start: Vector3i,
        end: Vector3i,
    ) -> anyhow::Result<Vec<usize>> {
        let mut selection = vec![];

        // Dupe check for safety - we should always return ranges within the bounds of the library
        let mut range_end = end.clone();
        range_end.x = min(range_end.x, self.size.x);
        range_end.y = min(range_end.y, self.size.y);
        range_end.z = min(range_end.z, self.size.z);

        for y in start.y..end.y {
            for x in start.x..end.x {
                for z in start.z..end.z {
                    let location = Vector3i { x, y, z };
                    let index = self.get_index(location);

                    match books.get(index) {
                        Some(book) => {
                            if book.is_checked_out() {
                                return Err(anyhow::anyhow!(
                                "Tried to select book that's already checked out. location: {}, index: {}, version: {}",
                                location,
                                index,
                                book.version(),
                            ));
                            }
                            selection.push(index)
                        }
                        None => continue,
                    }
                }
            }
        }

        Ok(selection)
    }

    // TODO: repeated in chunk.rs
    fn get_index(&self, location: Vector3i) -> usize {
        (location.y * (self.size.x * self.size.z) + location.x * self.size.z + location.z) as usize
    }

    fn now() -> String {
        let now = Utc::now();
        now.to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
    }
}
