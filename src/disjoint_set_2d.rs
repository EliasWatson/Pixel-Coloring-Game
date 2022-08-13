use crate::vec_2d::{Index2d, Vec2d};

#[derive(Debug, Clone)]
pub struct DisjointSet2d {
    group_count: usize,
    parents: Vec2d<Index2d>,
}

impl DisjointSet2d {
    pub fn from_vec_2d<T: Eq>(data: &Vec2d<T>) -> DisjointSet2d {
        let mut disjoint_set = DisjointSet2d {
            group_count: data.width * data.height,
            parents: Vec2d::generate(data.width, data.height, |x, y| (x, y)),
        };

        for y in 0..data.height {
            for x in 0..data.width {
                let current_value = match data.get((x, y)) {
                    Some(v) => v,
                    None => continue,
                };

                if let Some(right_value) = data.get((x + 1, y)) {
                    if *current_value == *right_value {
                        disjoint_set.link((x, y), (x + 1, y));
                    }
                }

                if let Some(down_value) = data.get((x, y + 1)) {
                    if *current_value == *down_value {
                        disjoint_set.link((x, y), (x, y + 1));
                    }
                }
            }
        }

        disjoint_set
    }

    pub fn get_parent(&self, index: Index2d) -> Option<Index2d> {
        let mut current_index = index;

        loop {
            let parent_index = *self.parents.get(current_index)?;

            if parent_index == current_index {
                break;
            }

            current_index = parent_index;
        }

        Some(current_index)
    }

    pub fn link(&mut self, a: Index2d, b: Index2d) {
        let a_parent = match self.get_parent(a) {
            Some(p) => p,
            None => return,
        };
        let b_parent = match self.get_parent(b) {
            Some(p) => p,
            None => return,
        };

        if a_parent == b_parent {
            return;
        }

        self.parents.set(a_parent, b_parent);
        self.group_count -= 1;
    }
}
