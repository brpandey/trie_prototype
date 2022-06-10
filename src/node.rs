use std::mem;
use std::str;
use std::collections::VecDeque;
use std::collections::HashMap;

use crate::iter::NodeDFSIter;
use crate::delete::{Playback, Cursor};

#[derive(Debug)]
pub struct Node {
    key: u8,
    value: Option<i32>,
    tag: NodeType,
    edges: HashMap<u8, Box<Node>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NodeType {
    Key,
    Inner,
}

#[derive(Debug, PartialEq)]
pub enum EdgeType {
    Single,
    Many,
}

pub type SearchResult<'a> = Option<(NodeType, &'a Node)>;
pub type NodeEdgesIter<'a> = std::collections::hash_map::Values<'a, u8, Box<Node>>;

impl Node {
    pub fn new(key: u8, value: Option<i32>) -> Self {
        Node {
            key,
            value,
            tag: NodeType::Inner,
            edges: HashMap::new(),
        }
    }

    pub fn key(&self) -> &u8 {
        &self.key
    }

    pub fn is_key(&self) -> bool {
        self.tag == NodeType::Key
    }

    pub fn edge_type(&self) -> Option<EdgeType> {
        match self.edges.len() {
            0 => None,
            1 => Some(EdgeType::Single),
            _ => Some(EdgeType::Many),
        }
    }

    pub fn edges_iter(&self) -> NodeEdgesIter<'_> {
        self.edges.values()
    }

    pub fn search(&self, prefix: &str) -> SearchResult<'_> {
        let mut current: &Node = self;

        // Iterate through key str while checking if each level of node contains
        // a matching prefix character. If we've searched through all characters
        // and the last node is a key node then success, return a ref to this node
        for &b in prefix.as_bytes().iter() {
            current =
                match current.edges.get(&b) {
                    Some(value) => &**value,
                    None => return None,
                };
        }

        return Some((current.tag, current))
    }

    pub fn longest_prefix(&self, prefix: &str) -> Option<String> {

        let mut stack: Vec<(&Node, u8, Option<u8>, u32)> = self.fold(prefix);
        let mut prefixes: Vec<u8>;

        // Iteratively pop off all stack elements until we find a value that is a NodeType::Key or not
        // If so, we prematurely drain the stack to collect all the remaining prefix tokens of the antecedents
        // and reasssemble this then longest prefix
        let mut result = None;

        while !stack.is_empty() {
            let (node, byte, _, _) = stack.pop().unwrap();

            if node.tag == NodeType::Key {
                prefixes = Vec::with_capacity(stack.len() + 1);
                prefixes = stack.drain(1..).fold(prefixes, |mut acc, (_, b, _, _)| {
                    acc.push(b);
                    acc}
                );

                prefixes.push(byte);
                result = Some(prefixes)
            }
        }

        result.map(|bytes| {
            String::from_utf8(bytes).unwrap()
        })
    }

    // Find all prefix keys which have the common prefix
    pub fn all_keys(&self, prefix: &str) -> Option<Vec<String>> {
        // Grab node where the prefix search ends
        let (_, current) = self.search(prefix)?;
        let mut result: Vec<String> = Vec::new();

        // VecDeque item is tuple: (reference to node, bytes prefix associate with node)
        let mut backlog: VecDeque<(&Node, Vec<u8>)> = VecDeque::new(); //initialize with tree capacity

        let mut node: &Node;
        let mut new_bytes: Vec<u8>;

        backlog.push_back((current, prefix.as_bytes().to_vec())); // seed queue

        while !backlog.is_empty() {
            let (current, bytes) = backlog.pop_front().unwrap();

            for v in current.edges.values() {
                node = &**v;
                new_bytes = bytes.clone();

                // update the prefix token for this node,
                // store along with node ref in backlog
                new_bytes.push(node.key);
                backlog.push_back((node, new_bytes))
            }

            if current.tag == NodeType::Key {
                result.push(String::from_utf8(bytes).unwrap())
            }
        }

        Some(result)
    }

    // If value already present return it and replace it
    // If value not already present, insert it creating new intermediate
    // nodes as necessary
    pub fn insert(&mut self, prefix: &str, value: Option<i32>) -> Option<i32> {
        let mut current: &mut Node = self;
        let mut temp: &mut Box<Node>;

        if prefix.is_empty() {
            return None
        }

        for &byte in prefix.as_bytes().iter() {
            temp = current.edges.entry(byte).or_insert_with(|| {
                Box::new(Node::new(byte, None))
            });

            current = &mut **temp;
        }

        // As we have finished iterating through, the prefix mark the node properly
        // if a node is marked already as a Key Node, (indicating it was previously
        // inserted), grab old value out and replace  with new boxed node)
        match current.tag {
            NodeType::Inner => {
                current.tag = NodeType::Key;
                current.value = value;
                None
            },
            NodeType::Key => {
                let old_node = mem::replace(current, Node::new(current.key, value));
                let _old_edge = mem::replace(&mut current.edges, old_node.edges);
                current.tag = NodeType::Key;
                old_node.value
            }
        }
    }

    pub fn fold(&self, prefix: &str) -> Vec<(&'_ Node, u8, Option<u8>, u32)> {
        let mut stack: Vec<(&Node, u8, Option<u8>, u32)> = Vec::new();

        let mut current: &Node = self;
        let mut node: &Node;
        let mut level: u32 = 0;
        let prefix_iter = &mut prefix.as_bytes().iter().peekable();

        let mut peek_byte = prefix_iter.peek().cloned().cloned();

        // Seed node stack with root node and first byte of prefix as the next byte
        stack.push((current, current.key, peek_byte, level));

        // Populate the stack by iterating through prefix bytes and each matching node level
        while let Some(&b) = prefix_iter.next() {
            level += 1;

            current =
                match current.edges.get(&b) {
                    Some(value) => {
                        node = &**value;
                        peek_byte = prefix_iter.peek().cloned().cloned();
                        stack.push((node, node.key, peek_byte, level));
                        node
                    },
                    None => {
                        break
                    }
                };
        }

        stack
    }

    pub fn remove(&mut self, prefix: &str) -> Option<i32> {
        let mut replay = crate::delete::capture(&self, prefix);

        let mut current: &mut Node = self;
        let mut item: Playback;
        let mut counter: u32 = 0;
        let mut temp: &mut Box<Node>;
        let mut value: Option<i32> = None;
        let mut removed;

        // As long as replay plan isn't empty follow the plan
        while !replay.is_empty() {
            item = replay.pop().unwrap();

            match item {
                Playback::Keep(Cursor::Link(i, byte, next_byte)) if i == counter && byte == current.key => {
                    temp = current.edges.get_mut(&next_byte).unwrap();
                    current = &mut **temp;
                },
                Playback::Prune(Cursor::Link(i, byte, next_byte)) if i == counter && byte == current.key => {
                    removed = current.edges.remove(&next_byte).unwrap();
                    current = &mut *removed;
                },
                Playback::Unmark(Cursor::Node(i, byte)) if byte == current.key && i == counter => {
                    current.tag = NodeType::Inner;
                    value = current.value.take();

                },
                _ => {
                    unreachable!()
                }
            }

            counter += 1;
        }

        value
    }

    
}

impl Node {
    pub(crate) fn iter(&self) -> NodeDFSIter<'_> {
        NodeDFSIter::new(self)
    }
}

impl <'a> IntoIterator for &'a Node {
    type Item = &'a u8;
    type IntoIter = NodeDFSIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

