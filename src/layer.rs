use std::ops::{Index, IndexMut};

// Generic layer interface
pub struct Layer<T: Default> {
    pub width: u32,
    pub height: u32,

    elements: Vec<Vec<T>>,
}

impl<T: Default> Layer<T> {
    pub fn new(width: u32, height: u32) -> Self {
        let mut elements = Vec::with_capacity(width as usize);
        for x in 0..width {
            elements.push(Vec::with_capacity(height as usize));

            for _ in 0..height {
                elements[x as usize].push(T::default());
            }
        }

        Self {
            width,
            height,
            elements,
        }
    }

    pub fn get(&self, x: isize, y: isize) -> Option<&T> {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            Some(&self.elements[x as usize][y as usize])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut T> {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            Some(&mut self.elements[x as usize][y as usize])
        } else {
            None
        }
    }
}

impl<T: Default> Index<(u32, u32)> for Layer<T> {
    type Output = T;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self.elements[index.0 as usize][index.1 as usize]
    }
}

impl<T: Default> IndexMut<(u32, u32)> for Layer<T> {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        &mut self.elements[index.0 as usize][index.1 as usize]
    }
}
