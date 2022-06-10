use std::iter::Peekable;
use crate::node::Node;
use crate::node::NodeEdgesIter;


type ItemsIter<'a> = Peekable<Box<NodeEdgesIter<'a>>>;

#[derive(Debug)]
enum NodeTraverse<'a>{
    Item(&'a Node),
    Iter(ItemsIter<'a>),
}

pub struct NodeDFSIter<'a> {
    current: Option<NodeTraverse<'a>>,
    unvisited: Vec<NodeTraverse<'a>>,
}

impl<'a> NodeDFSIter<'a> {
    pub fn new(node: &'a Node) -> NodeDFSIter<'a> {
        NodeDFSIter {
            current: Some(NodeTraverse::Item(node)),
            unvisited: Vec::new(),
        }
    }
}


// NodeDFSIter methods

impl<'a> NodeDFSIter<'a> {
    fn add_iter(&mut self, mut iter: ItemsIter<'a>) {
        if let Some(n) = iter.next() {
            self.current = Some(NodeTraverse::Item(n));
            if let Some(_) = iter.peek() {
                self.unvisited.push(NodeTraverse::Iter(iter))
            }
        }
    }
}

impl<'a> Iterator for NodeDFSIter<'a> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<Self::Item> {
        let iter: ItemsIter;

        loop {
            match self.current.take() {
                None => match self.unvisited.pop() {
                    Some(last) => self.current = Some(last),
                    None => break None,
                },
                Some(NodeTraverse::Item(n)) => {
                    iter = Box::new(n.edges_iter()).peekable();
                    self.add_iter(iter);
                    break Some(n.key())
                },
                Some(NodeTraverse::Iter(iter)) => {
                    self.add_iter(iter)
                }
            }
        }
    }
}
