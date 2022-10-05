use std::ops::Deref;

/// A vec that stores a 'current' index, which can be incremented and decremented, and loops around at the vec bounds
pub struct LoopVec<T> {
    index: usize,
    vec: Vec<T>,
}

impl<T> Default for LoopVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for LoopVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> FromIterator<T> for LoopVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            index: 0,
            vec: iter.into_iter().collect(),
        }
    }
}

impl<T> LoopVec<T> {
    pub fn new() -> Self {
        Self {
            index: 0,
            vec: Vec::new(),
        }
    }

    /// Gets the element at the current index, or `None` if the vec is empty
    pub fn get_current(&self) -> Option<&T> {
        self.vec.get(self.index)
    }

    pub fn get_current_mut(&mut self) -> Option<&mut T> {
        self.vec.get_mut(self.index)
    }

    /// Gets the current index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Increments the current index, looping around to 0 if incrementing exceeds the bounds of the vec
    pub fn inc(&mut self) {
        self.index = (self.index + 1)
            .checked_rem(self.len())
            .unwrap_or(self.index);
    }

    /// Decrements the current index, looping around to the tail of the vec if decrementing puts the index below 0
    pub fn dec(&mut self) {
        self.index = self.index.checked_sub(1).unwrap_or(self.len() - 1);
    }
}
