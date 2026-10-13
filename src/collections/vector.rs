pub trait VecExt<T> {
    fn find_index<F>(&self, predicate: F) -> Option<usize> where F: FnMut(&T) -> bool;

    fn count_matches<F>(&self, predicate: F) -> usize where F: FnMut(&T) -> bool;

    fn remove_first(&mut self, value: &T) -> bool where T: PartialEq;

    fn indices_of<F>(&self, predicate: F) -> Vec<usize> where F: FnMut(&T) -> bool;
}

impl<T> VecExt<T> for Vec<T> {
    fn find_index<F>(&self, predicate: F) -> Option<usize> where F: FnMut(&T) -> bool {
        self.iter().position(predicate)
    }

    fn count_matches<F>(&self, mut predicate: F) -> usize where F: FnMut(&T) -> bool {
        self.iter()
            .filter(|item| predicate(item))
            .count()
    }

    fn remove_first(&mut self, value: &T) -> bool where T: PartialEq {
        if let Some(index) = self.iter().position(|item| item == value) {
            self.remove(index);
            true
        } else {
            false
        }
    }
    fn indices_of<F>(&self, mut predicate: F) -> Vec<usize> where F: FnMut(&T) -> bool {
        self.iter()
            .enumerate()
            .filter_map(|(index, item)| predicate(item).then_some(index))
            .collect()
    }
}
