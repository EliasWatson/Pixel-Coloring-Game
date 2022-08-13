pub type Index2d = (usize, usize);

#[derive(Debug, Default, Clone)]
pub struct Vec2d<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Vec<T>>,
}

impl<T: Clone> Vec2d<T> {
    pub fn fill(width: usize, height: usize, val: T) -> Self {
        Vec2d {
            width,
            height,
            data: vec![vec![val; width]; height],
        }
    }
}

impl<T> Vec2d<T> {
    pub fn generate(width: usize, height: usize, generator: fn(usize, usize) -> T) -> Self {
        Vec2d {
            width,
            height,
            data: (0..height)
                .map(|y| (0..width).map(|x| generator(x, y)).collect())
                .collect(),
        }
    }

    pub fn get(&self, index: Index2d) -> Option<&T> {
        self.data.get(index.1)?.get(index.0)
    }

    pub fn set(&mut self, index: Index2d, value: T) {
        let row = match self.data.get_mut(index.1) {
            Some(r) => r,
            None => return,
        };

        if index.0 < row.len() {
            row[index.0] = value;
        }
    }
}
