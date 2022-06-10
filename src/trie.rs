use crate::node::Node;
use crate::node::SearchResult;

#[derive(Debug)]
pub struct Trie {
    size: u32,
    root: Node,
}

impl Trie {
    pub fn new() -> Self {
        Trie { size: 0, root: Node::new(Default::default(), None) }
    }

    pub fn search(&self, token: &str) -> SearchResult<'_> {
        self.root.search(token)
    }

    pub fn insert(&mut self, token: &str, value: Option<i32>) -> Option<i32> {
        let result = self.root.insert(token, value);

        if result.is_none() {
            self.size += 1
        }

        result
    }

    pub fn longest_prefix(&self, token: &str) -> Option<String> {
        self.root.longest_prefix(token)
    }

    pub fn all_keys(&self, token: &str) -> Option<Vec<String>> {
        self.root.all_keys(token)
    }

    pub fn remove(&mut self, token: &str) -> Option<i32> {
        self.root.remove(token)
    }

    pub fn keys(&self) -> String {
        let result = self.root.iter().copied().collect::<Vec<u8>>();
        let result = String::from_utf8(result).unwrap();
        println!("keys are {:?}", result);
        result
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}



