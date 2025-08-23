use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
};

use crate::{apriori::AprioriCounting, transaction_id::TransactionIdCounts};
/// A Hash Tree for the Apriori Algorithm
#[derive(Debug, Default)]
pub struct AprioriHashTree(AprioriHashTreeGeneric<50>);

impl AprioriHashTree {
    pub fn new() -> Self {
        Self(AprioriHashTreeGeneric::<50>::default())
    }
}
impl Deref for AprioriHashTree {
    type Target = AprioriHashTreeGeneric<50>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for AprioriHashTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
/// A Hash Tree for the Apriori Algorithm
/// Does not care about duplicates.
#[derive(Debug, Default)]
pub struct AprioriHashTreeGeneric<const N: usize> {
    root: HashTreeInternalNode<N>,
    /// The number of elements in the Tree
    length: usize,
}

impl<const N: usize> AprioriHashTreeGeneric<N> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Gets the leaf node for v
    fn get_leaf(&self, v: &[usize]) -> Option<&HashTreeLeafNode> {
        assert!(!v.is_empty());
        // Gets the node for the first element
        let mut hasher = DefaultHasher::new();
        v[0].hash(&mut hasher);
        let mut curr = &self.root.map[(hasher.finish() as usize) % N];
        // Loops through the rest of v
        for &v in v.iter().skip(1) {
            // Unwrap the node
            if let Some(n) = curr {
                // Check if an internal node exists for it.
                match n.as_ref() {
                    // Set the current node to the next node
                    Node::Internal(hash_tree_internal_node) => {
                        let mut hasher = DefaultHasher::new();
                        v.hash(&mut hasher);
                        curr = &hash_tree_internal_node.map[(hasher.finish() as usize) % N];
                    }
                    // If a leaf is there, then too many elements in v
                    Node::Leaf(_) => return None,
                }
            } else {
                return None;
            }
        }
        // Checks if curr is some Node
        if let Some(n) = curr {
            match n.as_ref() {
                // If it's an internal, then v has too little elements
                Node::Internal(_) => return None,
                // Otherwise it is good
                Node::Leaf(hash_tree_leaf_node) => return Some(hash_tree_leaf_node),
            }
        }
        None
    }
    /// Gets the leaf node of the tree with a mutable reference
    fn get_leaf_mut(&mut self, v: &[usize]) -> Option<&mut HashTreeLeafNode> {
        assert!(!v.is_empty());
        // Gets the first node from v[0]
        let mut hasher = DefaultHasher::new();
        v[0].hash(&mut hasher);
        let mut curr = &mut self.root.map[(hasher.finish() as usize) % N];
        // Loop through the rest of the elements for v
        for v in v.iter().skip(1) {
            // Unwraps the current node
            if let Some(n) = curr {
                match n.as_mut() {
                    // If it's an internal node, get the next node
                    Node::Internal(hash_tree_internal_node) => {
                        let mut hasher = DefaultHasher::new();
                        v.hash(&mut hasher);
                        curr = &mut hash_tree_internal_node.map[(hasher.finish() as usize) % N];
                    }
                    // Otherwise v is too big
                    Node::Leaf(_) => return None,
                }
            } else {
                return None;
            }
        }
        // Unwrap the current node
        if let Some(n) = curr {
            match n.as_mut() {
                // If it's internal, v is too small
                Node::Internal(_) => return None,
                // Otherwise, it is good
                Node::Leaf(hash_tree_leaf_node) => return Some(hash_tree_leaf_node),
            }
        }
        None
    }
    /// Checks if the tree contains v
    pub fn contains(&self, v: &[usize]) -> bool {
        assert!(!v.is_empty());
        let leaf = self.get_leaf(v);
        if let Some(l) = leaf {
            l.contains(v)
        } else {
            false
        }
    }
    /// Adds v to the tree
    pub fn add(&mut self, v: &[usize]) {
        assert!(!v.is_empty());
        // Gets the node for v[0]
        let mut hasher = DefaultHasher::new();
        v[0].hash(&mut hasher);
        let hash = hasher.finish() as usize;
        let mut curr = &mut self.root.map[hash % N];
        // Loops through the rest of v
        for v in v.iter().skip(1) {
            // If the node does not exist, create a new internal node
            if curr.is_none() {
                *curr = Some(Box::new(Node::Internal(HashTreeInternalNode::default())));
            }
            if let Some(n) = curr {
                match n.as_mut() {
                    // If n is an internal node, get the next node
                    Node::Internal(hash_tree_internal_node) => {
                        let mut hasher = DefaultHasher::new();
                        v.hash(&mut hasher);
                        curr = &mut hash_tree_internal_node.map[(hasher.finish() as usize) % N];
                    }
                    // Otherwise v is too big
                    Node::Leaf(_) => return,
                }
            }
        }
        // Create a leaf node if curr is none
        if curr.is_none() {
            *curr = Some(Box::new(Node::Leaf(HashTreeLeafNode::default())));
        }
        if let Some(n) = curr {
            match n.as_mut() {
                // If curr is an internal node, v is too small
                Node::Internal(_) => return,
                Node::Leaf(hash_tree_leaf_node) => hash_tree_leaf_node.add(v),
            }
        }
        // Increment the length because we added an element
        self.length += 1;
    }
    /// Increments v
    pub fn increment(&mut self, v: &[usize]) -> bool {
        assert!(!v.is_empty());
        let leaf = self.get_leaf_mut(v);
        if let Some(leaf) = leaf {
            leaf.increment(v);
            true
        } else {
            false
        }
    }
    /// Gets the count of v
    pub fn get_count(&self, v: &[usize]) -> Option<u64> {
        let leaf = self.get_leaf(v);
        if let Some(l) = leaf {
            l.get_count(v)
        } else {
            None
        }
    }
    /// Removes v from the tree
    pub fn remove(&mut self, v: &[usize]) -> Option<(Vec<usize>, u64)> {
        let leaf = self.get_leaf_mut(v);
        if let Some(l) = leaf {
            l.remove(v)
        } else {
            None
        }
    }
    /// A for each mutable loop (Mutable iterator is too much of a pain to write)
    pub fn for_each_mut(&mut self, mut f: impl FnMut(&[usize], &mut u64)) {
        self.root.for_each_mut(&mut f);
    }
    /// Gets an iterator for the Hash Tree
    pub fn iter(&self) -> HashTreeIterator<N> {
        HashTreeIterator::new(self)
    }
    /// Gets the number of elements in the tree
    pub fn len(&self) -> usize {
        self.length
    }
    /// Checks if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
/// An Enum for a node of the Hash Tree.
#[derive(Debug)]
enum Node<const N: usize> {
    Internal(HashTreeInternalNode<N>),
    Leaf(HashTreeLeafNode),
}
/// The internal node for the Hash Tree
#[derive(Debug)]
struct HashTreeInternalNode<const N: usize> {
    /// A mapping to its children nodes
    map: [Option<Box<Node<N>>>; N],
}

impl<const N: usize> HashTreeInternalNode<N> {
    /// A mutable for each loop
    fn for_each_mut(&mut self, f: &mut impl FnMut(&[usize], &mut u64)) {
        for n in &mut self.map {
            let Some(n) = n else { continue };
            match &mut **n {
                // If the child is internal, then recursively call for_each_mut
                Node::Internal(hash_tree_internal_node) => hash_tree_internal_node.for_each_mut(f),
                // Otherwise loop through the leaf node
                Node::Leaf(hash_tree_leaf_node) => hash_tree_leaf_node.for_each_mut(f),
            }
        }
    }
}

impl<const N: usize> Default for HashTreeInternalNode<N> {
    fn default() -> Self {
        Self {
            map: [const { None }; N],
        }
    }
}
/// A Leaf node for the Hash Tree, which is just a Vector
#[derive(Debug, Default)]
struct HashTreeLeafNode(Vec<(Vec<usize>, u64)>);

impl HashTreeLeafNode {
    /// Increments at v
    fn increment(&mut self, v: &[usize]) -> bool {
        let f = self.0.iter_mut().find(|v2| v2.0.eq(v));
        if let Some(v) = f {
            v.1 += 1;
            true
        } else {
            false
        }
    }
    /// Gets the element at v
    fn find(&self, v: &[usize]) -> Option<&(Vec<usize>, u64)> {
        self.0.iter().find(|v2| v2.0.eq(v))
    }
    /// Mutable For Each Loop
    fn for_each_mut(&mut self, f: &mut impl FnMut(&[usize], &mut u64)) {
        for (v, n) in &mut self.0 {
            f(v, n);
        }
    }
    /// Gets the mutable element at v
    fn find_mut(&mut self, v: &[usize]) -> Option<&mut (Vec<usize>, u64)> {
        self.0.iter_mut().find(|v2| v2.0.eq(v))
    }
    /// Checks if self contains v
    fn contains(&self, v: &[usize]) -> bool {
        self.find(v).is_some()
    }
    /// Add v to the Leaf
    fn add(&mut self, v: &[usize]) {
        self.0.push((v.to_vec(), 0));
    }
    /// Gets the count at v
    fn get_count(&self, v: &[usize]) -> Option<u64> {
        self.find(v).map(|f| f.1)
    }
    /// Removes v from the Leaf
    fn remove(&mut self, v: &[usize]) -> Option<(Vec<usize>, u64)> {
        for i in 0..self.0.len() {
            if v.eq(self.0[i].0.as_slice()) {
                return Some(self.0.remove(i));
            }
        }
        None
    }
}
/// The Hash Tree Iterator
pub struct HashTreeIterator<'a, const N: usize> {
    tree: &'a AprioriHashTreeGeneric<N>,
    /// The current index for the first internal node
    outer: usize,
    /// The stack for the iterations
    stack: Vec<(&'a Node<N>, usize)>,
}

impl<'a, const N: usize> Iterator for HashTreeIterator<'a, N> {
    type Item = (&'a [usize], u64);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If the stack is empty, do the root
            if self.stack.is_empty() {
                let mut i = self.outer;
                // Increments i until we find a non-empty spot
                while i < N && self.tree.root.map[i].is_none() {
                    i += 1;
                }
                if i >= N {
                    return None;
                }
                // The next index to look at
                self.outer = i + 1;
                match &self.tree.root.map[i] {
                    // Push the non-empty node to the stack
                    Some(a) => self.stack.push((a.as_ref(), 0)),
                    None => unreachable!(),
                }
            }
            while !self.stack.is_empty() {
                // Gets the last index used
                let mut i = self.stack.last().unwrap().1;
                match self.stack.last().unwrap().0 {
                    // If the last node was an internal node
                    Node::Internal(hash_tree_internal_node) => {
                        // Increment until we find some element
                        while i < N && hash_tree_internal_node.map[i].is_none() {
                            i += 1;
                        }
                        // If i is too large, then we have iterated through everything
                        if i >= N {
                            self.stack.pop();
                            continue;
                        }
                        self.stack.last_mut().unwrap().1 = i + 1;
                        // Add the next node to the stack
                        match &hash_tree_internal_node.map[i] {
                            Some(a) => self.stack.push((a, 0)),
                            None => unreachable!(),
                        }
                    }
                    Node::Leaf(hash_tree_leaf_node) => {
                        // We have iterator through everything on the leaf
                        if i >= hash_tree_leaf_node.0.len() {
                            self.stack.pop();
                            continue;
                        }
                        self.stack.last_mut().unwrap().1 += 1;
                        // Return the element at the leaf
                        return Some((&hash_tree_leaf_node.0[i].0, hash_tree_leaf_node.0[i].1));
                    }
                }
            }
        }
    }
}

impl<'a, const N: usize> HashTreeIterator<'a, N> {
    fn new(tree: &'a AprioriHashTreeGeneric<N>) -> Self {
        Self {
            tree,
            stack: Vec::new(),
            outer: 0,
        }
    }
}
impl<const N: usize> TransactionIdCounts for AprioriHashTreeGeneric<N> {
    fn increment(&mut self, v: &[usize]) -> bool {
        self.increment(v)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn for_each(&self, mut f: impl FnMut(&[usize])) {
        self.iter().for_each(|v| f(v.0));
    }
}

impl<const N: usize> AprioriCounting for AprioriHashTreeGeneric<N> {
    fn len(&self) -> usize {
        self.len()
    }

    fn increment(&mut self, v: &[usize]) -> bool {
        self.increment(v)
    }

    fn for_each_mut(&mut self, f: impl FnMut(&[usize], &mut u64)) {
        self.for_each_mut(f);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::hash_tree::AprioriHashTreeGeneric;

    use super::AprioriHashTree;

    #[test]
    fn test_hash_tree() {
        let mut tree = AprioriHashTree::default();
        tree.add(&[1, 2]);
        assert!(tree.contains(&[1, 2]));
        tree.increment(&[1, 2]);
        assert_eq!(tree.get_count(&[1, 2]), Some(1));
        assert!(!tree.contains(&[1, 3]));
        assert_eq!(tree.get_count(&[1, 3]), None);
        assert_eq!(tree.remove(&[1, 2]), Some((vec![1, 2], 1)));
        assert!(!tree.contains(&[1, 2]));
    }
    #[test]
    fn test_hash_tree_iterator() {
        let mut tree = AprioriHashTreeGeneric::<2>::default();
        tree.add(&[1, 2]);
        tree.increment(&[1, 2]);
        tree.add(&[1, 3]);
        let mut set = HashSet::new();
        set.insert([1, 2]);
        set.insert([1, 3]);
        for item in tree.iter() {
            assert!(set.remove(item.0));
        }
        assert!(set.is_empty());
    }
}
