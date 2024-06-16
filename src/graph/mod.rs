use std::{
    iter::{zip, Map},
    ptr,
    rc::{Rc, Weak},
};

use crate::power_system::PsEdge;

pub mod transform;
pub mod plague_algo;

#[derive(Debug, Clone, Copy)]
pub enum GraphError {
    NodeIndexWrongGraph(usize),
    EdgeIndexWrongGraph(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    TO,
    FROM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdjacentInfo {
    pub edge_index: EdgeIndex,
    pub node_index: NodeIndex,
    pub dir: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    pub adjacent: Vec<AdjacentInfo>,
    pub index: NodeIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeInfo {
    pub fnode: NodeIndex,
    pub tnode: NodeIndex,
    pub index: EdgeIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeIndex(pub usize);

#[derive(Debug, Clone)]
pub struct Node<'g, N> {
    pub data: &'g N,
    pub info: &'g NodeInfo,
}

#[derive(Debug, Clone)]
pub struct Edge<'g, E> {
    pub data: &'g E,
    pub info: &'g EdgeInfo,
}

#[derive(Debug)]
pub struct Graph<N, E> {
    nodes: Vec<NodeInfo>,
    edges: Vec<EdgeInfo>,
    pub node_data: Vec<N>,
    pub edge_data: Vec<E>,
}

impl<N, E> Graph<N, E> {
    pub fn empty_graph() -> Graph<N, E> {
        return Graph {
            nodes: vec![],
            edges: vec![],
            node_data: vec![],
            edge_data: vec![]
        };
    }

    pub fn add_node(&mut self, node_data: N) -> NodeIndex {
        let node_index = NodeIndex(self.node_data.len());

        self.node_data.push(node_data);

        self.nodes.push(NodeInfo {
            adjacent: vec![],
            index: node_index,
        });

        node_index
    }

    pub fn add_all_nodes(&mut self, node_data: Vec<N>) -> Vec<NodeIndex> {
        node_data.into_iter().map(|nd| self.add_node(nd)).collect()
    }

    pub fn add_edge(&mut self, edge_data: E, fnode_index: NodeIndex, tnode_index: NodeIndex) -> EdgeIndex {
        let edge_index = self.edge_data.len();
        self.edge_data.push(edge_data);

        self.nodes[fnode_index.0].adjacent.push(AdjacentInfo {
            edge_index: EdgeIndex(edge_index),
            node_index: tnode_index,
            dir: Direction::FROM,
        });
        self.nodes[tnode_index.0].adjacent.push(AdjacentInfo {
            edge_index: EdgeIndex(edge_index),
            node_index: fnode_index,
            dir: Direction::TO,
        });

        self.edges.push(EdgeInfo {
            fnode: fnode_index,
            tnode: tnode_index,
            index: EdgeIndex(edge_index),
        });

        return EdgeIndex(edge_index);
    }

    pub fn get_node(&self, node_index: NodeIndex) -> Node<'_, N> {
        Node {
            data: &self.node_data[node_index.0],
            info: &self.nodes[node_index.0],
        }
    }

    pub fn get_node_count(&self) -> usize{
        self.node_data.len()
    }

    pub fn get_adjacency_info(&self, node_index: NodeIndex) -> &Vec<AdjacentInfo> {
        &self.nodes[node_index.0].adjacent
    }

    pub fn get_edge(&self, edge_index: EdgeIndex) -> Edge<'_, E> {
        Edge {
            data: &self.edge_data[edge_index.0],
            info: &self.edges[edge_index.0],
        }
    }

    pub fn get_edge_data(&self, edge_index: EdgeIndex) -> &E {
        &self.edge_data[edge_index.0]
    }

    pub fn transform<NT, ET, N2, E2>(&self, node_transform: NT, edge_transform: ET) -> Graph<N2, E2>
    where
        NT: Fn(&N) -> N2,
        ET: Fn(&E) -> E2,
    {
        let mut graph_new = Graph::<N2, E2>::empty_graph();

        let new_node_data = self
            .node_data
            .iter()
            .map(node_transform)
            .collect();
        graph_new.add_all_nodes(new_node_data);

        zip(
            self.edge_data.iter().map(edge_transform),
            self.edges.iter(),
        )
        .for_each(|(transformed_edge_data, edge_info)| {
            graph_new.add_edge(transformed_edge_data, edge_info.fnode, edge_info.tnode);
        });

        return graph_new;
    }

    pub fn loop_count(&self) -> usize {
        self.edges.iter().filter(|ed| ed.fnode == ed.tnode).count()
    }

    pub fn connected_to(&self, edge_index: EdgeIndex, node_index: NodeIndex) -> bool {
        let edge = self.edges[edge_index.0];
        edge.fnode == node_index || edge.tnode == node_index
    }

    pub fn edges(&self) -> Vec<Edge<'_, E>> {
        return zip(&self.edge_data, &self.edges)
        .map(|(data, info)| Edge {
            data: data,
            info: info
        }).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_graph() {
        let mut g_mut = Graph::<String, String>::empty_graph();

        let node_index1 = g_mut.add_node(String::from("node1"));
        let node_index2 = g_mut.add_node(String::from("node2"));

        let edge_index1 = g_mut
            .add_edge(String::from("edge1"), node_index1, node_index2);

        let g = g_mut;

        let node1 = g.get_node(node_index1);
        assert!(node1.data == &String::from("node1"));
        assert!(node1.info.index == NodeIndex(0));
        assert!(node1.info.adjacent.len() == 1);
        assert!(
            node1.info.adjacent[0]
                == AdjacentInfo {
                    edge_index: edge_index1,
                    node_index: node_index2,
                    dir: Direction::FROM
                }
        );

        let node2 = g.get_node(node_index2);

        assert!(node2.data == &String::from("node2"));
        assert!(node2.info.index == node_index2);
        assert!(node2.info.adjacent.len() == 1);
        assert!(
            node2.info.adjacent[0]
                == AdjacentInfo {
                    edge_index: edge_index1,
                    node_index: node_index1,
                    dir: Direction::TO
                }
        );

        let edge = g.get_edge(edge_index1);

        assert!(edge.data == &String::from("edge1"));
        assert!(edge.info.index == EdgeIndex(0));
        assert!(edge.info.tnode == NodeIndex(1));
        assert!(edge.info.fnode == NodeIndex(0));
    }

    mod tests {
        use super::*;
    
        #[test]
        fn isomorphic_transform_test() {
            let mut g_mut = Graph::<String, String>::empty_graph();
    
            let node_index1 = g_mut.add_node(String::from("node1"));
            let node_index2 = g_mut.add_node(String::from("node2"));
    
            let edge_index1 = g_mut.add_edge(String::from("edge1"), node_index1, node_index2);
    
            let g_start = g_mut;
    
            let g = g_start.transform(|nd| nd.to_uppercase(), |ed| ed.to_uppercase());
    
            let node1 = g.get_node(node_index1);
            assert!(node1.data == &String::from("NODE1"));
            assert!(node1.info.index == node_index1);
            assert!(node1.info.adjacent.len() == 1);
            assert!(
                node1.info.adjacent[0]
                    == AdjacentInfo {
                        edge_index: edge_index1,
                        node_index: node_index2,
                        dir: Direction::FROM
                    }
            );
    
            let node2 = g.get_node(node_index2);
    
            assert!(node2.data == &String::from("NODE2"));
            assert!(node2.info.index == node_index2);
            assert!(node2.info.adjacent.len() == 1);
            assert!(
                node2.info.adjacent[0]
                    == AdjacentInfo {
                        edge_index: edge_index1,
                        node_index: node_index1,
                        dir: Direction::TO
                    }
            );
    
            let edge = g.get_edge(edge_index1);
    
            assert!(edge.data == &String::from("EDGE1"));
            assert!(edge.info.index == edge_index1);
            assert!(edge.info.tnode == NodeIndex(1));
            assert!(edge.info.fnode == NodeIndex(0));
        }
    }
}
