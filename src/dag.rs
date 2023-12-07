use glam::IVec2;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub position: IVec2,
}

impl Node {
    pub fn with_kind(kind: NodeKind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }

    pub fn positioned(mut self, position: IVec2) -> Self {
        self.position = position;
        self
    }

    pub fn inputs(&self) -> InputIterator {
        self.kind.inputs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeKind {
    Passthrough(u32),
    Intrinsic(Intrinsic),
    Input,
    Constant(f32),
}

impl NodeKind {
    pub fn inputs(&self) -> InputIterator {
        InputIterator { kind: *self, i: 0 }
    }
}

impl Default for NodeKind {
    fn default() -> Self {
        Self::Passthrough(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intrinsic {
    Add(u32, u32),
    Sub(u32, u32),
    Mul(u32, u32),
    Div(u32, u32),
}

pub struct InputIterator {
    kind: NodeKind,
    i: usize,
}

impl Iterator for InputIterator {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let out = match (self.kind, self.i) {
            (NodeKind::Passthrough(input), 0) => Some(input),
            (NodeKind::Intrinsic(Intrinsic::Add(a, _)), 0) => Some(a),
            (NodeKind::Intrinsic(Intrinsic::Add(_, b)), 1) => Some(b),
            (NodeKind::Intrinsic(Intrinsic::Sub(a, _)), 0) => Some(a),
            (NodeKind::Intrinsic(Intrinsic::Sub(_, b)), 1) => Some(b),
            (NodeKind::Intrinsic(Intrinsic::Mul(a, _)), 0) => Some(a),
            (NodeKind::Intrinsic(Intrinsic::Mul(_, b)), 1) => Some(b),
            (NodeKind::Intrinsic(Intrinsic::Div(a, _)), 0) => Some(a),
            (NodeKind::Intrinsic(Intrinsic::Div(_, b)), 1) => Some(b),
            _ => None,
        };
        self.i += 1;
        out
    }
}

#[derive(Default)]
pub struct Dag {
    out_node: u32,
    next_node: u32,
    nodes: HashMap<u32, Node>,
}

impl Dag {
    pub fn new() -> Self {
        Self {
            out_node: 0,
            next_node: 1,
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> u32 {
        let id = self.next_node;
        self.next_node += 1;
        self.nodes.insert(id, node);
        id
    }

    pub fn remove_vertex(&mut self, node: u32) {
        self.nodes.remove(&node);
    }

    pub fn add_input(&mut self, node: u32, input: u32, index: usize) -> Result<(), EdgeError> {
        if node == input {
            return Err(EdgeError::SameNode);
        }

        if self.reachable(input, node) {
            return Err(EdgeError::CreatesCycle);
        }

        let node = self.nodes.get_mut(&node).ok_or(EdgeError::MissingNode)?;

        match node.kind {
            NodeKind::Constant(_) | NodeKind::Input => Err(EdgeError::InputIndex),
            NodeKind::Passthrough(_) => {
                if index == 0 {
                    node.kind = NodeKind::Passthrough(input);
                    Ok(())
                } else {
                    Err(EdgeError::InputIndex)
                }
            }

            NodeKind::Intrinsic(intrinsic) => {
                node.kind = NodeKind::Intrinsic(match (intrinsic, index) {
                    (Intrinsic::Add(_, b), 0) => Intrinsic::Add(input, b),
                    (Intrinsic::Add(a, _), 1) => Intrinsic::Add(a, input),
                    (Intrinsic::Sub(_, b), 0) => Intrinsic::Sub(input, b),
                    (Intrinsic::Sub(a, _), 1) => Intrinsic::Sub(a, input),
                    (Intrinsic::Mul(_, b), 0) => Intrinsic::Mul(input, b),
                    (Intrinsic::Mul(a, _), 1) => Intrinsic::Mul(a, input),
                    (Intrinsic::Div(_, b), 0) => Intrinsic::Div(input, b),
                    (Intrinsic::Div(a, _), 1) => Intrinsic::Div(a, input),
                    _ => return Err(EdgeError::InputIndex),
                });
                Ok(())
            }
        }
    }

    pub fn set_out_node(&mut self, node: u32) {
        assert!(self.nodes.keys().any(|&id| id == node));
        self.out_node = node;
    }

    pub fn out_node(&self) -> u32 {
        self.out_node
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
            if let Some(src) = self.node(src) {
                for neighbor in src.inputs() {
                    if self.reachable_inner(neighbor, dst, visited) {
                        return true;
                    }
                }
            }
            false
        }
    }

    pub fn node(&self, id: u32) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u32, &Node)> {
        self.nodes.iter()
    }

    pub fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.nodes.keys().cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EdgeError {
    #[error("The vertex already exists")]
    MissingNode,
    #[error("The edge already exists")]
    ExistingEdge,
    #[error("Adding the edge creates a cycle")]
    CreatesCycle,
    #[error("Cannot connect a node to itself")]
    SameNode,
    #[error("The node input index is out of bounds")]
    InputIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_cycle() {
        let mut dag = Dag::new();
        let a = dag.add_node(Node::default());
        let b = dag.add_node(Node::default());
        let c = dag.add_node(Node::default());
        assert_eq!(dag.add_input(a, b, 0), Ok(()));
        assert_eq!(dag.add_input(b, c, 0), Ok(()));
        assert_eq!(dag.add_input(c, a, 0), Err(EdgeError::CreatesCycle));
    }
}
