
pub struct Node<'g, N, E>{
    edges: Vec<&'g Edge<'g, N, E>>,
    index: NodeIndex,
    data: &'g N,
}

pub struct NodeIndex(usize);

pub struct Edge<'g, N, E>{
    fnode: &'g Node<'g, N, E>,
    tnode: &'g Node<'g, N, E>,
    index: EdgeIndex,
    data: &'g E,
}

pub struct EdgeIndex(usize);

pub struct Graph<'g, N, E>{
    pub nodes: Vec<Node<'g, N, E>>,
    pub edges: Vec<Edge<'g, N, E>>,
    pub node_data: Vec<N>,
    pub edge_data: Vec<E>,
}

impl<'g, N, E> Graph<'g, N, E> {

    pub fn new() -> Graph<'g, N, E> {
        return Graph { nodes: vec![], edges: vec![], node_data: vec![], edge_data: vec![]}
    }

    pub fn add_node(&mut self, node_data: N) -> NodeIndex {
        let index = NodeIndex(self.node_data.len());

        self.node_data.push(
            node_data
        );

        self.nodes.push(
            Node { edges:vec![], index: index, data: &self.node_data[index.0] }
        );

        index
    }

    pub fn add_edge_safe(&mut self, edge_data: E, tnode_index: &NodeIndex, fnode_index: &NodeIndex) -> Option<EdgeIndex> {
        if tnode_index.0 >= self.node_data.len() || fnode_index.0 >= self.node_data.len() {
            return None;
        } else {
            
            let index = EdgeIndex(self.edge_data.len());

            self.edge_data.push(
                edge_data
            );

            self.edges.push(
                Edge {
                    tnode: &self.nodes[tnode_index.0],
                    fnode: &self.nodes[fnode_index.0],
                    index: index,
                    data: &self.edge_data[index.0],
                }
            );

            return Some(index);
        }
    }

}
