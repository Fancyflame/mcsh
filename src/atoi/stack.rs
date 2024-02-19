#[derive(Clone, Debug)]
pub(super) struct UnsizedStack<'a, T> {
    items: Vec<(&'a str, T)>,
    block_indexes: Vec<usize>,
}

impl<'a, T> UnsizedStack<'a, T> {
    pub fn new() -> Self {
        UnsizedStack {
            items: Vec::new(),
            block_indexes: Vec::new(),
        }
    }

    pub fn push(&mut self, tag: &'a str, item: T) {
        if self.block_indexes.is_empty() {
            self.delimite();
        }
        self.items.push((tag, item))
    }

    pub fn delimite(&mut self) {
        self.block_indexes.push(self.items.len());
    }

    pub fn find_newest(&self, tag: &str) -> Option<&T> {
        self.items
            .iter()
            .rev()
            .find(|(t, _)| *t == tag)
            .map(|(_, item)| item)
    }

    pub fn has_sibling_namesake(&self, tag: &str) -> bool {
        let start = self.block_indexes.last().copied().unwrap_or_default();
        self.items[start..].iter().any(|(t, _)| *t == tag)
    }

    pub fn pop_block(&mut self) {
        if let Some(size) = self.block_indexes.pop() {
            self.items.truncate(size);
        }
    }
}
