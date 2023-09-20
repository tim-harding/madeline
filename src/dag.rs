use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Node {
    kind: NodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeKind {
    #[default]
    Null,
}

#[derive(Default)]
struct Dag {
    next_node: u32,
    adjacency: HashMap<u32, Vec<u32>>,
    nodes: HashMap<u32, Node>,
}

impl Dag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vertex(&mut self, node: Node) -> u32 {
        let node = self.next_node;
        self.next_node += 1;
        self.adjacency.entry(node).or_insert(vec![]);
        node
    }

    pub fn add_edge(&mut self, src: u32, dst: u32) -> Result<(), EdgeError> {
        if self.reachable(dst, src) {
            return Err(EdgeError::CreatesCycle);
        }

        if !self.adjacency.contains_key(&dst) {
            return Err(EdgeError::MissingVertex(dst));
        }

        let src_adjacency = self
            .adjacency
            .get_mut(&src)
            .ok_or(EdgeError::MissingVertex(src))?;

        if src_adjacency.contains(&dst) {
            return Err(EdgeError::ExistingEdge);
        }
        src_adjacency.push(dst);
        Ok(())
    }

    pub fn reachable(&self, src: u32, dst: u32) -> bool {
        let mut visited = HashSet::new();
        self.reachable_inner(src, dst, &mut visited)
    }

    fn reachable_inner(&self, src: u32, dst: u32, visited: &mut HashSet<u32>) -> bool {
        if src == dst {
            true
        } else if visited.contains(&src) {
            false
        } else {
            visited.insert(src);
            for neighbor in self.adjacency.get(&src).unwrap() {
                if self.reachable_inner(*neighbor, dst, visited) {
                    return true;
                }
            }
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EdgeError {
    #[error("The vertex {0} already exists")]
    MissingVertex(u32),
    #[error("The edge already exists")]
    ExistingEdge,
    #[error("Adding the edge creates a cycle")]
    CreatesCycle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_cycle() {
        let mut dag = Dag::new();
        let a = dag.add_vertex(Node::default());
        let b = dag.add_vertex(Node::default());
        let c = dag.add_vertex(Node::default());
        assert_eq!(dag.add_edge(a, b), Ok(()));
        assert_eq!(dag.add_edge(b, c), Ok(()));
        assert_eq!(dag.add_edge(c, a), Err(EdgeError::CreatesCycle));
    }
}
