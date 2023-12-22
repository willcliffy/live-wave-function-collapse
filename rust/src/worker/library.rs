use godot::builtin::Vector3i;

pub trait Book {
    fn location(&self) -> Vector3i;

    fn version(&self) -> String;
    fn set_version(&mut self, version: String);

    fn is_checked_out(&self) -> bool;
    fn check_out(&mut self) -> bool;
}

pub struct Library3D<T> {
    size: Vector3i,
    books: Vec<T>,
}

impl<T: Book + Clone> Library3D<T> {
    pub fn new(size: Vector3i, books: Vec<T>) -> Self {
        Self { size, books }
    }

    // TODO - this is NOT atomic, and should be
    pub fn check_out_range(&mut self, start: Vector3i, end: Vector3i) -> Result<Vec<T>, String> {
        let mut books = vec![];

        for y in start.y..end.y {
            for x in start.x..end.x {
                for z in start.z..end.z {
                    let location = Vector3i { x, y, z };
                    let index = self.get_index(location);
                    match self.get_book_mut(index) {
                        Some(book) => {
                            if book.is_checked_out() {
                                return Err(format!(
                                    "Tried to check out book that's already checked out {}",
                                    location
                                ));
                            }

                            book.check_out();
                            books.push(book.clone())
                        }
                        None => {
                            return Err(format!(
                                "Tried to check out book that doesn't exist: {}",
                                location
                            ));
                        }
                    }
                }
            }
        }

        Ok(books)
    }

    // TODO - this is NOT atomic, and should be
    pub fn check_in_range(&mut self, books: Vec<T>, version: String) -> Result<(), String> {
        for book in books.iter() {
            match self.check_in_book(book, version.clone()) {
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn check_in_book(&mut self, book: &T, version: String) -> Result<(), String> {
        let index = self.get_index(book.location());

        match self.get_book(index) {
            Some(current_book) => {
                if current_book.version() != book.version() {
                    return Err(format!("Version mismatch on check-in: tried to check in from version {} but newest version is {}", book.version(), current_book.version()));
                }
            }
            None => {
                return Err(format!(
                    "Invalid index on check-in: index {} (for position {}) is out of bounds {}",
                    index,
                    book.location(),
                    self.books.len()
                ))
            }
        }

        self.set_book(index, book, version)
    }

    fn set_book(&mut self, index: usize, book: &T, version: String) -> Result<(), String> {
        if index >= self.books.len() {
            return Err(format!(
                "Failed to set book: index {} (for position {}) is out of bounds {}",
                index,
                book.location(),
                self.books.len()
            ));
        }

        let mut book = book.clone();
        book.set_version(version);
        self.books[index] = book;
        Ok(())
    }

    fn get_book(&self, index: usize) -> Option<&T> {
        self.books.get(index)
    }

    fn get_book_mut(&mut self, index: usize) -> Option<&mut T> {
        self.books.get_mut(index)
    }

    // TODO: repeated in chunk.rs
    fn get_index(&self, location: Vector3i) -> usize {
        (location.y * (self.size.x * self.size.z) + location.x * self.size.z + location.z) as usize
    }
}
