use std::{
    collections::{HashMap, HashSet},
    io::repeat,
    iter,
};

use crate::power_system::EdgeData;

use super::*;

#[derive(Debug)]
pub struct SubGraphMap {
    to_subgraph_node: Vec<Option<NodeIndex>>,
    to_subgraph_edge: Vec<Option<EdgeIndex>>,
    to_supergraph_nodes: Vec<Vec<NodeIndex>>,
    to_supergraph_edge: Vec<EdgeIndex>,
}

impl SubGraphMap {
    fn new(supergraph_node_size: usize, supergraph_edge_size: usize) -> Self {
        SubGraphMap {
            to_subgraph_node: iter::repeat(None).take(supergraph_node_size).collect(),
            to_subgraph_edge: iter::repeat(None).take(supergraph_edge_size).collect(),
            to_supergraph_nodes: vec![],
            to_supergraph_edge: vec![],
        }
    }

    fn add_node_indices(&mut self, to_supergraph_nodes: &Vec<Vec<NodeIndex>>) {
        // self.to_supergraph_nodes = super_indices.iter().map(|ni| vec![ni.clone()]).collect();

        // self.to_supergraph_nodes
        //     .iter()
        //     .enumerate()
        //     .for_each(|(sub_index, super_index)| {
        //         self.to_subgraph_node[super_index[0].0] = Some(NodeIndex(sub_index));
        //                     });

        // for (super_node_index, ele) in merge_indices.iter().enumerate() {
        //     if self.to_subgraph_node[super_node_index].is_some() {
        //         let sub_node = self.to_subgraph_node[super_node_index].unwrap();
        //         self.to_supergraph_nodes[sub_node.0].extend(ele);
        //     }
        // }

        self.to_supergraph_nodes = to_supergraph_nodes.to_vec();

        for (sub_index, super_indices) in self.to_supergraph_nodes.iter().enumerate() {
            for super_index in super_indices.iter() {
                self.to_subgraph_node[super_index.0] = Some(NodeIndex(sub_index));
            }
        }

        
    }

    fn add_edge_indices(&mut self, super_indices: Vec<EdgeIndex>) {
        self.to_supergraph_edge = super_indices;

        self.to_supergraph_edge
            .iter()
            .enumerate()
            .for_each(|(sub_index, super_index)| {
                self.to_subgraph_edge[super_index.0] = Some(EdgeIndex(sub_index));
            });
    }

    pub fn get_super_node(&self, sub_index: NodeIndex) -> &Vec<NodeIndex> {
        &self.to_supergraph_nodes[sub_index.0]
    }

    pub fn get_sub_node(&self, super_index: NodeIndex) -> Option<NodeIndex> {
        self.to_subgraph_node[super_index.0]
    }

    pub fn get_super_edge(&self, sub_index: EdgeIndex) -> EdgeIndex {
        self.to_supergraph_edge[sub_index.0]
    }

    pub fn get_sub_edge(&self, super_index: EdgeIndex) -> Option<EdgeIndex> {
        self.to_subgraph_edge[super_index.0]
    }
}

pub struct CreateSubGraph<'a, N, E>
where
    N: Clone,
    E: Clone,
{
    old_graph: &'a Graph<N, E>,
    new_graph: Graph<N, E>,

    to_subgraph_node: Vec<Option<NodeIndex>>,
    merged_indices: Vec<Vec<NodeIndex>>,
    to_subgraph_edge: Vec<Option<EdgeIndex>>,

    nodes: Vec<NodeInfo>,
    edges: Vec<EdgeInfo>,
    node_data: Vec<N>,
    edge_data: Vec<E>,
}

impl<'a, N, E> CreateSubGraph<'a, N, E>
where
    N: Clone,
    E: Clone,
{
    pub fn new<NT, NF, ET>(
        old_graph: &Graph<N, E>,
        node_transform: NT,
        node_filter: NF,
        edge_transform: ET,
    ) -> CreateSubGraph<'_, N, E>
    where
        NT: Fn(&N) -> N,
        NF: Fn(&N) -> bool,
        ET: Fn(&E) -> E,
    {
        let mut new_graph = CreateSubGraph {
            old_graph,
            new_graph: Graph::<N, E>::empty_graph(),
            to_subgraph_node: vec![],
            merged_indices: vec![],
            to_subgraph_edge: vec![],
            nodes: vec![],
            edges: vec![],
            node_data: vec![],
            edge_data: vec![],
        };

        new_graph.init_new_graph(node_transform, node_filter, edge_transform);

        return new_graph;
    }

    fn init_new_graph<NT, NF, ET>(
        &mut self,
        node_transform: NT,
        node_filter: NF,
        edge_transform: ET,
    ) where
        NT: Fn(&N) -> N,
        NF: Fn(&N) -> bool,
        ET: Fn(&E) -> E,
    {
        self.node_data = self
            .old_graph
            .node_data
            .iter()
            .map(node_transform)
            .collect::<Vec<N>>();

        self.edge_data = self
            .old_graph
            .edge_data
            .iter()
            .map(edge_transform)
            .collect::<Vec<E>>();

        self.to_subgraph_node = self
            .old_graph
            .node_data
            .iter()
            .enumerate()
            .map(|(index, nd)| {
                if node_filter(nd) {
                    Some(NodeIndex(index))
                } else {
                    None
                }
            })
            .collect::<Vec<Option<NodeIndex>>>();

        self.merged_indices = self.to_subgraph_node.iter().enumerate().map(|(index, node)| match node {
            Some(_) => vec![NodeIndex(index)],
            None => vec![],
        }).collect();

        self.to_subgraph_edge = self
            .old_graph
            .edges
            .iter()
            .map(|ei| {
                if self.to_subgraph_node[ei.tnode.0].is_some()
                    && self.to_subgraph_node[ei.fnode.0].is_some()
                {
                    Some(EdgeIndex(ei.index.0))
                } else {
                    None
                }
            })
            .collect::<Vec<Option<EdgeIndex>>>();

        self.nodes = self.old_graph.nodes.iter().map(|n| n.clone()).collect();
        self.edges = self.old_graph.edges.iter().map(|e| e.clone()).collect();

        // let blah = 0.0;
    }

    pub fn edge_contraction_filter<NM, EF>(&mut self, node_merge: &NM, edge_filter: &EF)
    where
        NM: Fn(&E, &N, &N) -> N,
        EF: Fn(&E) -> bool,
    {
        let indices_to_merge = zip(&self.to_subgraph_edge, &self.edge_data)
            .enumerate()
            .filter(|(_index, (sub_edge, e))| sub_edge.is_some() && edge_filter(e))
            .map(|(index, (_sub_edge, _e))| EdgeIndex(index))
            .collect::<Vec<EdgeIndex>>();

        indices_to_merge.iter().for_each(|ei| {
            self.edge_contraction(*ei, node_merge);
        });
    }

    pub fn edge_contraction<NM>(&mut self, edge_index: EdgeIndex, node_merge: &NM)
    where
        NM: Fn(&E, &N, &N) -> N,
    {
        if self.to_subgraph_edge[edge_index.0].is_none() {
            return;
        } else if self.edges[edge_index.0].tnode == self.edges[edge_index.0].fnode {
            self.to_subgraph_edge[edge_index.0] = None;

            let node_index = self.edges[edge_index.0].tnode.0;
            self.node_data[node_index] = node_merge(
                &self.edge_data[edge_index.0],
                &self.node_data[node_index],
                &self.node_data[node_index],
            );
        } else {
            let edge_info: EdgeInfo = self.edges[self.to_subgraph_edge[edge_index.0].unwrap().0];
            // println!("{:?}", edge_info);
            self.to_subgraph_edge[edge_index.0] = None;

            let node_to_keep = edge_info.tnode;
            let node_to_discard = edge_info.fnode;
            // println!("{:?} -> {:?}", node_to_discard, node_to_keep);
            self.to_subgraph_node[node_to_discard.0] = None;
            let clon = self.merged_indices[node_to_discard.0].clone();
            self.merged_indices[node_to_keep.0].extend(clon);
            self.merged_indices[node_to_discard.0] = vec![];

            // self.nodes.iter().for_each(|n| {
            //     n.adjacent.iter_mut().for_each(|ajc| {
            //         if ajc.node_index == 
            //     })
            // });

            self.nodes[node_to_keep.0].adjacent = self.nodes[node_to_keep.0]
                .adjacent
                .iter()
                .chain(self.nodes[node_to_discard.0].adjacent.iter())
                .filter(|adj_info| adj_info.edge_index != edge_index)
                .map(|adj_info| adj_info.clone())
                .collect();

            let adjacenies_to_fix = self.nodes[node_to_discard.0]
                .adjacent
                .iter()
                // .filter(|adj_info| adj_info.edge_index != edge_index)
                .map(|adj_info| adj_info.clone())
                .collect::<Vec<AdjacentInfo>>();

                // println!("adjacenices {:?}", adjacenies_to_fix);
                // println!("");

            adjacenies_to_fix.iter().for_each(|adj| {
                self.nodes[adj.node_index.0]
                    .adjacent
                    .iter_mut()
                    .for_each(|adj_info_to_remain| {
                        adj_info_to_remain.node_index = node_to_keep;
                    });

                if self.edges[adj.edge_index.0].tnode == node_to_discard {
                    self.edges[adj.edge_index.0].tnode = node_to_keep
                }

                if self.edges[adj.edge_index.0].fnode == node_to_discard {
                    self.edges[adj.edge_index.0].fnode = node_to_keep
                }
            });
            
            self.node_data[node_to_keep.0] = node_merge(
                &self.edge_data[edge_index.0],
                &self.node_data[node_to_discard.0],
                &self.node_data[node_to_keep.0],
            );

            // println!("Edges {:?}", self.edges);
            // println!("");
            // println!("Edges dead {:?}", self.to_subgraph_edge);
            // println!("");
            // println!("=========");
        }
    }

    pub fn complete(mut self) -> (Graph<N, E>, SubGraphMap) {
        let mut g = Graph::<N, E>::empty_graph();

        // println!("to_subgraph_node {:?}", self.to_subgraph_node);
        // println!("to_subgraph_node {:?}", self.to_subgraph_node);

        // println!("merge_indies {:?}", self.merged_indices);

        self.merged_indices.iter().enumerate().for_each(|(sub_index, super_indices)|{
            super_indices.iter().for_each(|super_index| {
                // println!("self.to_subgraph_node {:?}", self.to_subgraph_node);
                self.to_subgraph_node[super_index.0] = Some(NodeIndex(sub_index));
            })
        });

        let to_supergraph_nodes = self.merged_indices.iter().filter(|v| v.len() > 0 ).map(|v| v.clone()).collect();

        // println!("self.to_subgraph_node {:?}", self.to_subgraph_node);

        let mut subgraph_map = SubGraphMap::new(
            self.old_graph.get_node_count(),
            self.old_graph.edge_data.len(),
        );

        subgraph_map.add_node_indices(
            &to_supergraph_nodes
        );

        subgraph_map.add_edge_indices(
            self.to_subgraph_edge
                .iter()
                .filter(|opt_index| opt_index.is_some())
                .map(|opt_index| opt_index.unwrap())
                .collect(),
        );

        let contracted_nodes = self.merged_indices
        .iter()
        .enumerate()
        .filter(|(_index, vec)| vec.len() > 0)
        .map(|(index, _vec)| self.node_data[index].clone()).collect::<Vec<N>>();

        g.add_all_nodes(contracted_nodes);

        self.edge_data
            .iter()
            .enumerate()
            .for_each(|(super_index, ed)| {
                if self.to_subgraph_edge[super_index].is_none() {
                    //do nothing
                } else {
                    let edge_info = self.edges[super_index];
                    let sub_fnode = subgraph_map.get_sub_node(edge_info.fnode).unwrap();
                    let sub_tnode = subgraph_map.get_sub_node(edge_info.tnode).unwrap();

                    g.add_edge(ed.clone(), sub_fnode, sub_tnode);
                }
            });

        (g, subgraph_map)
    }
}

mod tests {
    use std::{clone, convert::identity};

    use super::*;

    #[test]
    fn create_graph() {
        let mut g_mut = Graph::<String, String>::empty_graph();

        let node_index1 = g_mut.add_node(String::from("node1"));
        let node_index2 = g_mut.add_node(String::from("node2"));
        let node_index3 = g_mut.add_node(String::from("node3"));

        let edge_index32 = g_mut.add_edge(String::from("edge32"), node_index3, node_index2);

        let edge_index13 = g_mut.add_edge(String::from("edge13"), node_index1, node_index3);

        let g = g_mut;

        let create_sub_graph = CreateSubGraph::<'_, String, String>::new(
            &g,
            |n| n.clone(),
            |n| n != "node2",
            |e| e.clone(),
        );

        let (pruned_graph, sub_graph_map) = create_sub_graph.complete();

        assert_eq!(pruned_graph.edges.len(), 1);
        assert_eq!(pruned_graph.nodes.len(), 2);

        assert_eq!(pruned_graph.node_data[0], "node1");
        assert_eq!(pruned_graph.node_data[1], "node3");
        assert_eq!(pruned_graph.edge_data[0], "edge13");

        assert!(sub_graph_map.get_sub_node(node_index2).is_none());
        assert!(sub_graph_map.get_sub_edge(edge_index32).is_none());

        let node1 = pruned_graph.get_node(sub_graph_map.get_sub_node(node_index1).unwrap());

        assert_eq!(node1.data, "node1");
        assert_eq!(
            node1.info.adjacent,
            vec![AdjacentInfo {
                edge_index: EdgeIndex(0),
                node_index: NodeIndex(1),
                dir: Direction::FROM,
            }]
        );

        let node3 = pruned_graph.get_node(sub_graph_map.get_sub_node(node_index3).unwrap());

        assert_eq!(node3.data, "node3");
        assert_eq!(
            node3.info.adjacent,
            vec![AdjacentInfo {
                edge_index: EdgeIndex(0),
                node_index: NodeIndex(0),
                dir: Direction::TO,
            }]
        );

        let edge3 = pruned_graph.get_edge(sub_graph_map.get_sub_edge(edge_index13).unwrap());
        assert_eq!(edge3.data, "edge13");
        assert_eq!(
            edge3.info,
            &EdgeInfo {
                fnode: sub_graph_map.get_sub_node(node_index1).unwrap(),
                tnode: sub_graph_map.get_sub_node(node_index3).unwrap(),
                index: sub_graph_map.get_sub_edge(edge_index13).unwrap()
            }
        );
    }

    #[test]
    fn contract_edge() {
        let mut g_mut = Graph::<String, String>::empty_graph();

        let node_index1 = g_mut.add_node(String::from("node1"));
        let node_index2 = g_mut.add_node(String::from("node2"));
        let node_index3 = g_mut.add_node(String::from("node3"));
        let node_index4 = g_mut.add_node(String::from("node4"));
        let node_index5 = g_mut.add_node(String::from("node5"));

        let _edge_index12 = g_mut.add_edge(String::from("edge12"), node_index1, node_index2);
        let edge_index23 = g_mut.add_edge(String::from("edge23"), node_index2, node_index3);
        let _edge_index34 = g_mut.add_edge(String::from("edge34"), node_index3, node_index4);
        let _edge_index15 = g_mut.add_edge(String::from("edge15"), node_index1, node_index5);
        let _edge_index25 = g_mut.add_edge(String::from("edge25"), node_index2, node_index5);

        let g = g_mut;

        let mut create_sub_graph =
            CreateSubGraph::<'_, String, String>::new(&g, |n| n.clone(), |_n| true, |e| e.clone());

        let nm = |e: &String, fnode: &String, tnode: &String| fnode.to_string() + e + tnode;
        create_sub_graph.edge_contraction(edge_index23, &nm);

        let (pruned_graph, sub_graph_map) = create_sub_graph.complete();

        println!("{:#?}", pruned_graph);

        assert_eq!(pruned_graph.edges.len(), 4);
        assert_eq!(pruned_graph.nodes.len(), 4);

        assert_eq!(pruned_graph.node_data[0], "node1");
        assert_eq!(pruned_graph.node_data[1], "node2edge23node3");
        assert_eq!(pruned_graph.edge_data[0], "edge12");
        assert_eq!(pruned_graph.edge_data[1], "edge34");

        assert!(sub_graph_map.get_sub_node(node_index2).is_none());
        assert!(sub_graph_map.get_sub_edge(edge_index23).is_none());

        let node1 = pruned_graph.get_node(sub_graph_map.get_sub_node(node_index1).unwrap());

        assert_eq!(node1.data, "node1");
        assert_eq!(
            node1.info.adjacent,
            vec![
                AdjacentInfo {
                    edge_index: EdgeIndex(0),
                    node_index: NodeIndex(1),
                    dir: Direction::FROM,
                },
                AdjacentInfo {
                    edge_index: EdgeIndex(2),
                    node_index: NodeIndex(3),
                    dir: Direction::FROM
                }
            ]
        );

        let node3 = pruned_graph.get_node(sub_graph_map.get_sub_node(node_index3).unwrap());

        assert_eq!(node3.data, "node2edge23node3");
        assert_eq!(
            node3.info.adjacent,
            vec![
                AdjacentInfo {
                    edge_index: EdgeIndex(0),
                    node_index: NodeIndex(0),
                    dir: Direction::TO,
                },
                AdjacentInfo {
                    edge_index: EdgeIndex(1),
                    node_index: NodeIndex(2),
                    dir: Direction::FROM,
                },
                AdjacentInfo {
                    edge_index: EdgeIndex(3),
                    node_index: NodeIndex(3),
                    dir: Direction::FROM
                }
            ]
        );

        // let edge_merge = pruned_graph.get_edge(sub_graph_map.get_sub_edge(edge_index23).unwrap());

        assert_eq!(node3.data, "node2edge23node3");
        assert_eq!(
            node3.info.adjacent,
            vec![
                AdjacentInfo {
                    edge_index: EdgeIndex(0),
                    node_index: NodeIndex(0),
                    dir: Direction::TO,
                },
                AdjacentInfo {
                    edge_index: EdgeIndex(1),
                    node_index: NodeIndex(2),
                    dir: Direction::FROM,
                },
                AdjacentInfo {
                    edge_index: EdgeIndex(3),
                    node_index: NodeIndex(3),
                    dir: Direction::FROM
                }
            ]
        );
    }

    #[test]
    fn remove_loop_edges () {
        let mut g_mut = Graph::<String, String>::empty_graph();

        let node_index1 = g_mut.add_node(String::from("node1"));
        let node_index2 = g_mut.add_node(String::from("node2"));
        let node_index3 = g_mut.add_node(String::from("node3"));

        let _edge_index12 = g_mut.add_edge(String::from("edge12"), node_index1, node_index2);
        let _edge_index22 = g_mut.add_edge(String::from("edge22"), node_index2, node_index2);
        let _edge_index23 = g_mut.add_edge(String::from("edge23"), node_index2, node_index3);

        let g = g_mut;

        let mut create_sub_graph =
        CreateSubGraph::<'_, String, String>::new(&g, |n| n.clone(), |_n| true, |e| e.clone());

        let nm = |_e: &String, fnode: &String, _tnode: &String| fnode.to_string();
        let ef = |ed: &String| ed.as_str() == "edge22";
        create_sub_graph.edge_contraction_filter(&nm, &ef);

        let (pruned_graph, _sub_graph_map) = create_sub_graph.complete();

        assert_eq!(pruned_graph.edges.len(), 2);
        assert_eq!(pruned_graph.nodes.len(), 3);

        assert_eq!(pruned_graph.node_data[0], "node1");
        assert_eq!(pruned_graph.node_data[1], "node2");
        assert_eq!(pruned_graph.node_data[2], "node3");
        assert_eq!(pruned_graph.edge_data[0], "edge12");
        assert_eq!(pruned_graph.edge_data[1], "edge23");
    }
}
