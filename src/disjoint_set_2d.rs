use bevy::utils::HashSet;

use crate::vec_2d::{Index2d, Vec2d};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DisjoinSetEntry {
    Parent(HashSet<Index2d>),
    Child(Index2d),
}

#[derive(Debug, Clone)]
pub struct DisjointSet2d {
    group_count: usize,
    parents: Vec2d<DisjoinSetEntry>,
}

impl DisjointSet2d {
    pub fn from_vec_2d<T: Eq>(data: &Vec2d<T>) -> DisjointSet2d {
        let mut disjoint_set = DisjointSet2d {
            group_count: data.width * data.height,
            parents: Vec2d::generate(data.width, data.height, |x, y| {
                DisjoinSetEntry::Parent(HashSet::from([(x, y)]))
            }),
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
            match self.parents.get(current_index)? {
                DisjoinSetEntry::Parent(_) => return Some(current_index),
                DisjoinSetEntry::Child(parent_index) => current_index = *parent_index,
            }
        }
    }

    pub fn get_linked(&self, index: Index2d) -> Option<&HashSet<Index2d>> {
        match self.parents.get(self.get_parent(index)?)? {
            DisjoinSetEntry::Parent(set) => Some(set),
            DisjoinSetEntry::Child(_) => None,
        }
    }

    pub fn link(&mut self, a: Index2d, b: Index2d) {
        let a_parent_index = match self.get_parent(a) {
            Some(p) => p,
            None => return,
        };
        let b_parent_index = match self.get_parent(b) {
            Some(p) => p,
            None => return,
        };

        if a_parent_index == b_parent_index {
            return;
        }

        let a_set = match self.parents.get(a_parent_index) {
            Some(DisjoinSetEntry::Parent(set)) => set,
            _ => return,
        };

        let b_set = match self.parents.get(b_parent_index) {
            Some(DisjoinSetEntry::Parent(set)) => set,
            _ => return,
        };

        self.parents.set(
            a_parent_index,
            DisjoinSetEntry::Parent(a_set.union(b_set).copied().collect()),
        );

        self.parents
            .set(b_parent_index, DisjoinSetEntry::Child(a_parent_index));

        self.group_count -= 1;
    }
}
