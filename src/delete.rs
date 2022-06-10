use crate::node::{Node, EdgeType};

type DeletePlan = Vec<Playback>;

#[derive(Debug, PartialEq)]
pub enum Cursor {
    Node(u32, u8),
    Link(u32, u8, u8),
}

#[derive(Debug, PartialEq)]
pub enum Playback {
    Unmark(Cursor),
    Prune(Cursor),
    Keep(Cursor),
}


// In order to delete a node without using back or parent links we create a replay stack which
// gives us the required info to delete a node or prune nodes while iterating a single mutable pointer
// starting from the trie root node
// (While Rust supports recursion but not tail recursion this explicit stack somewhat likens
// to a call stack with no limitations of potentially blowing the call stack)
pub fn capture(node: &Node, prefix: &str) -> DeletePlan {
    let mut prune = false;
    let mut replay: Vec<Playback> = Vec::new();

    // Essentially reduce the multiple node immutable refs stack into
    // a replay stack which just has copy values / semantics
    let mut stack: Vec<(&Node, u8, Option<u8>, u32)> = node.fold(prefix);
    //prepopulated stack given prefix and trie

    if let Some((node_ref, b, _bn, level)) = stack.pop() {
        if node_ref.is_key() {
            replay.push(Playback::Unmark(Cursor::Node(level, b)));

            // No child edges then can easily prune
            if node_ref.edge_type().is_none() {
                prune = true
            }
        }
    }

    while !stack.is_empty() {
        let (node_ref, byte, bn, level) = stack.pop().unwrap();
        let byte_next = bn.unwrap();

        if prune {
            // We can only prune a level above the node that needs deleting
            let status = Cursor::Link(level, byte, byte_next);
            let item = Playback::Prune(status);
            replay.push(item);

            // If this node is a key node or has other edges, don't prune
            // Edge type has to be some since we're atleast the second node from the end
            if node_ref.is_key() || node_ref.edge_type().unwrap() == EdgeType::Many {
                prune = false
            }
        } else {
            replay.push(Playback::Keep(Cursor::Link(level, byte, byte_next)))
        }
    }

    replay
}
