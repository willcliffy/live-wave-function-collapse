use std::{
    cmp::min,
    sync::{Mutex, MutexGuard},
};

use chrono::Utc;
use godot::builtin::Vector3i;

const DIRECTIONS: &'static [Vector3i] = &[
    Vector3i::UP,
    Vector3i::DOWN,
    Vector3i::RIGHT,
    Vector3i::LEFT,
    Vector3i::FORWARD,
    Vector3i::BACK,
];

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
    pub start: Vector3i,
    pub end: Vector3i,
    pub books: Vec<T>,
}

impl<T> Range<T> {
    pub fn new(start: Vector3i, end: Vector3i, books: Vec<T>) -> Self {
        let size = end - start;
        Self {
            size,
            start,
            end,
            books,
        }
    }

    pub fn index(&self, location: Vector3i) -> usize {
        ((location.y - self.start.y) * (self.size.x * self.size.z)
            + (location.x - self.start.x) * self.size.z
            + (location.z - self.start.z)) as usize
    }

    // Returns true iff the given position is located within this range
    pub fn contains(&self, position: Vector3i) -> bool {
        position.x >= self.start.x
            && position.x < self.end.x
            && position.y >= self.start.y
            && position.y < self.end.y
            && position.z >= self.start.z
            && position.z < self.end.z
    }

    // Get all neighboring cells that are exactly one unit away, measured using Manhattan distance
    // That is, only check the 6 cardinal directions directly adjacent to cell_position
    // Diagonal cells are not returned. Cells that are not within this range are not returned.
    pub fn get_neighbors(&self, position: Vector3i) -> Vec<Vector3i> {
        let mut neighbors = vec![];
        for direction in DIRECTIONS {
            let neighbor_position = position + *direction;
            if self.contains(neighbor_position) {
                neighbors.push(neighbor_position);
            }
        }

        neighbors
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

        // TODO - slight perf bottleneck here
        // let start_time = Instant::now();
        match self.books.lock() {
            Ok(books_locked) => books = books_locked,
            Err(e) => return Err(anyhow::anyhow!("Failed to lock books: {}", e)),
        }
        // let duration = start_time.elapsed();
        // if duration.as_millis() > 20 {
        //     godot_print!("waited for lock for {:?}ms", duration.as_millis());
        // }

        let range_end = Vector3i {
            x: min(end.x, self.size.x),
            y: min(end.y, self.size.y),
            z: min(end.z, self.size.z),
        };

        let selection = self.get_selection(&books, start, range_end)?;

        let mut range_books = vec![];
        for i in selection {
            let book = books.get_mut(i).unwrap();
            book.check_out();
            range_books.push(book.clone())
        }

        Ok(Range::new(start, range_end, range_books))
    }

    pub fn check_in_range(&self, range: &mut Range<T>) -> anyhow::Result<()> {
        let mut books;

        // TODO - slight perf bottleneck here
        // let start_time = Instant::now();
        match self.books.lock() {
            Ok(books_locked) => books = books_locked,
            Err(e) => return Err(anyhow::anyhow!("Failed to lock books: {}", e)),
        }
        // let duration = start_time.elapsed();
        // if duration.as_millis() > 20 {
        //     godot_print!("waited for lock for {:?}ms", duration.as_millis());
        // }

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

    fn get_selection(
        &self,
        books: &MutexGuard<'_, Vec<T>>,
        start: Vector3i,
        end: Vector3i,
    ) -> anyhow::Result<Vec<usize>> {
        let mut selection = vec![];

        // Dupe check for safety - we should always return ranges within the bounds of the library
        let range_end = Vector3i {
            x: min(end.x, self.size.x),
            y: min(end.y, self.size.y),
            z: min(end.z, self.size.z),
        };

        for y in start.y..range_end.y {
            for x in start.x..range_end.x {
                for z in start.z..range_end.z {
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

    fn get_index(&self, location: Vector3i) -> usize {
        (location.y * (self.size.x * self.size.z) + location.x * self.size.z + location.z) as usize
    }

    fn now() -> String {
        let now = Utc::now();
        now.to_rfc3339_opts(chrono::SecondsFormat::Micros, false)
    }
}
