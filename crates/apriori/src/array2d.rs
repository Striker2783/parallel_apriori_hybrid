use std::collections::HashMap;

use parallel::traits::Convertable;

use crate::storage::AprioriCounter;

pub struct AprioriP2Counter2<'a> {
    arr: Array2D<u64>,
    reverse_map: HashMap<usize, usize>,
    map: &'a [usize],
}
impl<'a> AprioriP2Counter2<'a> {
    pub fn new(map: &'a [usize]) -> Self {
        let reverse_map = map
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, c)| (c, i))
            .collect();
        Self {
            arr: Array2D::new(map.len()),
            reverse_map,
            map,
        }
    }
}
impl AprioriCounter for AprioriP2Counter2<'_> {
    fn increment(&mut self, v: &[usize]) -> bool {
        if let (Some(a), Some(b)) = (
            self.reverse_map.get(&v[0]).cloned(),
            self.reverse_map.get(&v[1]).cloned(),
        ) {
            self.arr.increment(a, b);
            return true;
        }
        false
    }

    fn insert(&mut self, _: &[usize]) {
        unimplemented!()
    }

    fn get_count(&self, v: &[usize]) -> Option<u64> {
        if let (Some(a), Some(b)) = (
            self.reverse_map.get(&v[0]).cloned(),
            self.reverse_map.get(&v[1]).cloned(),
        ) {
            Some(self.arr.get(a, b))
        } else {
            None
        }
    }

    fn for_each(&self, mut f: impl FnMut(&[usize], u64)) {
        self.arr.iter().for_each(|(r, c, count)| {
            f(&[self.map[c], self.map[r]], count);
        });
    }

    fn len(&self) -> usize {
        self.arr.len()
    }
}

pub struct AprioriP2Counter(Array2D<u64>);
impl AprioriP2Counter {
    pub fn new(size: usize) -> Self {
        Self(Array2D::new(size))
    }
}
impl AprioriCounter for AprioriP2Counter {
    fn increment(&mut self, v: &[usize]) -> bool {
        self.0.increment(v[0], v[1]);
        true
    }

    fn insert(&mut self, _: &[usize]) {}

    fn get_count(&self, v: &[usize]) -> Option<u64> {
        Some(self.0.get(v[0], v[1]))
    }

    fn for_each(&self, mut f: impl FnMut(&[usize], u64)) {
        self.0.iter().for_each(|(r, c, count)| f(&[c, r], count));
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
/// A lower triangle 2D square matrix in the form of a 1D array.
#[derive(Debug, Default)]
pub struct Array2D<T>(Vec<T>);
impl<T: Copy> Array2D<T> {
    /// Gets the element at row and col
    pub fn get(&self, row: usize, col: usize) -> T {
        self.0[self.get_index(row, col)]
    }
}
impl<T: Copy + Default> Array2D<T> {
    /// Constructor with the number of rows
    pub fn new(rows: usize) -> Self {
        Array2D(vec![T::default(); (rows * (rows - 1)) / 2])
    }
}
impl<T> Array2D<T> {
    /// Gets the index into the 1D array based on row and col
    fn get_index(&self, row: usize, col: usize) -> usize {
        assert!(row != col);
        // The row must be greater than column
        let (row, col) = if row > col { (row, col) } else { (col, row) };
        let index = (row * (row - 1)) / 2 + col;
        assert!(index < self.0.len());
        index
    }
    /// Sets value into the 2D array
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        let index = self.get_index(row, col);
        self.0[index] = value;
    }
    /// Iterator over all the element of the 2D array.
    pub fn iter(&self) -> Array2DIterator<T> {
        Array2DIterator::new(self)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl Array2D<u64> {
    /// Increments at row, col
    pub fn increment(&mut self, row: usize, col: usize) {
        let index = self.get_index(row, col);
        self.0[index] += 1;
    }
    /// Adds up the corresponding elements in the 2D Array
    /// Both arrays must have equal sizes.
    pub fn add_assign(&mut self, rhs: &Array2D<u64>) {
        assert!(self.0.len() == rhs.0.len());
        for i in 0..self.0.len() {
            self.0[i] += rhs.0[i];
        }
    }
}
impl Convertable for Array2D<u64> {
    fn to_vec(&mut self) -> Vec<u64> {
        self.0.clone()
    }

    fn add_from_vec(&mut self, v: &[u64]) {
        assert_eq!(self.0.len(), v.len());
        (0..v.len()).for_each(|i| {
            self.0[i] += v[i];
        });
    }
}
/// The Iterator for the 2D Array
#[derive(Debug)]
pub struct Array2DIterator<'a, T> {
    data: &'a Array2D<T>,
    /// The current row
    row: usize,
    /// The current column
    col: usize,
    /// The current index
    idx: usize,
}

impl<'a, T> Array2DIterator<'a, T> {
    /// Constructor
    fn new(data: &'a Array2D<T>) -> Self {
        Self {
            data,
            row: 1,
            col: 0,
            idx: 0,
        }
    }
}
impl<T: Copy> Iterator for Array2DIterator<'_, T> {
    type Item = (usize, usize, T);
    fn next(&mut self) -> Option<Self::Item> {
        // Iterated over everything
        if self.idx >= self.data.0.len() {
            return None;
        }
        // Gets the element at the current position
        let element = (self.row, self.col, self.data.0[self.idx]);
        // Increments the position
        self.idx += 1;
        self.col += 1;
        if self.col >= self.row {
            self.col = 0;
            self.row += 1;
        }
        Some(element)
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        array2d::{AprioriP2Counter2, Array2D},
        storage::AprioriCounter,
    };

    #[test]
    fn test_array2d() {
        let mut array2d = Array2D::new(3);
        array2d.increment(0, 1);
        assert_eq!(array2d.get(0, 1), 1);
        array2d.increment(1, 2);
        assert_eq!(array2d.get(1, 2), 1);
        let mut array2d = Array2D::new(5);
        array2d.increment(4, 3);
        array2d.increment(4, 3);
        assert_eq!(array2d.get(4, 3), 2);
        let mut array2d = Array2D::new(10);
        let mut count = 0;
        for i in 0..10 {
            for j in 0..i {
                array2d.set(i, j, count);
                count += 1;
            }
        }
        for i in 0..45 {
            assert_eq!(array2d.0[i], i as u64);
        }
        for (i, e) in array2d.iter().enumerate() {
            assert_eq!(e.2, i as u64);
            assert_eq!(array2d.get(e.0, e.1), e.2);
        }
    }
    #[test]
    fn test_apriori_pass2() {
        let mut counter = AprioriP2Counter2::new(&[1, 3, 5]);
        assert!(counter.increment(&[1, 3]));
        assert!(counter.increment(&[3, 5]));
        assert_eq!(counter.get_count(&[1, 3]), Some(1));
        counter.for_each(|v, c| {
            if v == [1, 3] || v == [3, 5] {
                assert_eq!(c, 1);
            } else if v == [1, 5] {
                assert_eq!(c, 0);
            } else {
                panic!()
            }
        });
    }
}
