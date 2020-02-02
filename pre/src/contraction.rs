use super::*;

/// return new generated shortcuts
pub fn calc_shortcuts(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
    dijkstra: &mut dijkstra::Dijkstra,
) -> (Vec<Way>, Vec<EdgeId>) {
    // dijkstra.avoid_node(node);
    let mut shortcuts = Vec::<Way>::new();
    let mut used_edges = Vec::<EdgeId>::new();
    // get node neighbors
    let source_edges: Vec<EdgeId> =
        graph_helper::get_down_edge_ids(node, &down_offset, &down_index);
    let target_edges: Vec<EdgeId> = graph_helper::get_up_edge_ids(node, &up_offset);

    for source_edge in source_edges {
        let source_node = edges[source_edge].source;
        for target_edge in &target_edges {
            let target_node = edges[*target_edge].target;
            let weight = edges[source_edge].weight + edges[*target_edge].weight;
            // simple improvement: dijkstra should get rid of it anyway
            if source_node == target_node {
                continue;
            }
            // prevent dijkstra from running on whole graph
            dijkstra.set_max_weight(weight);
            let shortest_path = dijkstra.find_path(source_node, target_node, up_offset, edges);
            // create new shortcut where found path is shortest
            if shortest_path.is_some() {
                let shortest_path = shortest_path.unwrap();
                if shortest_path.1 >= weight {
                    shortcuts.push(Way {
                        source: source_node,
                        target: target_node,
                        weight: weight,
                        // TODO calculate sensfull id
                        id: None,
                        contrated_previous: Some(source_edge),
                        contrated_next: Some(*target_edge),
                    });
                }
                used_edges.push(source_edge);
                used_edges.push(*target_edge);
            }
        }
    }
    return (shortcuts, used_edges);
}

/// return new generated shortcuts
pub fn contract_node(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
    mut dijkstra: &mut dijkstra::Dijkstra,
) {
    let (shortcuts, used_edges) = calc_shortcuts(
        node,
        &edges,
        &up_offset,
        &down_offset,
        &down_index,
        &mut dijkstra,
    );

    // TODO handle old edges

    // for edge_id in used_edges.iter().rev() {
    //     resulting_edges.push(edges[edge_id]);
    //     edges.remove(edge_id);
    // }
    // check what edges are remaining of contracted node in up and down graph
    // remove them as well?
    //
    // TODO remove node/edges and reduce edges from remaining graph
}

/// run full contraction
pub fn run_contraction(
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Way>,
    up_offset: &mut Vec<EdgeId>,
    down_offset: &mut Vec<EdgeId>,
    down_index: &mut Vec<EdgeId>,
) {
    // convert edges have indices
    edges
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, x)| x.id = Some(i));

    let mut resulting_edges = Vec::<Way>::new();

    // ordering
    let remaining_nodes: Vec<NodeId> = (0..nodes.len()).collect();

    let heuristic = ordering::calculate_heuristic(
        &remaining_nodes,
        &edges,
        &up_offset,
        &down_offset,
        &down_index,
        nodes.len(),
    );

    let local_minima = ordering::get_local_minima(&heuristic);

    println!(
        "local_minima at {:?} with {:?}",
        local_minima, heuristic[local_minima]
    );
    let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(nodes.len());

    // for node in remaining_nodes {
    //     contract_node(
    //         node,
    //         &edges,
    //         &up_offset,
    //         &down_offset,
    //         &down_index,
    //         &mut dijkstra,
    //     );
    // }

    // while:
    // (re)calculate heuristic
    // get all minimas
    // calculate independent set via local minimas
    //      pick local minimum
    //      mark all neighbors as invalid
    //      ...
    // contract all valid nodes
    // collect shortcuts
    // rebuild graph with new shortcuts

    /*
    contraction parallel:
    Update Priorities of all Nodes with Simulated Contractions
    while Remaining Graph not Empty do
        I ← Independent Node Set
        E ← Necessary Shortcuts
        Move I to their Level
        Insert E into Remaining graph
        Update Priority of Neighbors of I with Simulated Contractions
    end while
    */
    // TODO convert back to usual ways wightout indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_shortcuts_no_witness() {
        // 0 -> 2 -> 3
        // 1 ->/ \-> 4
        let amount_nodes = 5;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 2, 1, 0));
        edges.push(Way::test(1, 2, 2, 1));
        edges.push(Way::test(2, 3, 3, 2));
        edges.push(Way::test(2, 4, 1, 3));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let (shortcuts, _used_edges) = calc_shortcuts(
            2,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &mut dijkstra,
        );

        let expected_shortcuts = vec![
            Way::shortcut(1, 3, 5, 1, 2),
            Way::shortcut(1, 4, 3, 1, 3),
            Way::shortcut(0, 3, 4, 0, 2),
            Way::shortcut(0, 4, 2, 0, 3),
        ];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn calc_shortcuts_witness() {
        // 0 -> 1 -> 2
        //  \-> 3 ->/
        let amount_nodes = 4;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 1, 0));
        edges.push(Way::test(1, 2, 1, 2));
        edges.push(Way::test(0, 3, 1, 1));
        edges.push(Way::test(3, 2, 1, 3));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let (shortcuts, _used_edges) = calc_shortcuts(
            1,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &mut dijkstra,
        );

        let expected_shortcuts = vec![Way::shortcut(0, 2, 2, 0, 2)];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn calc_shortcuts_witness_via_center() {
        // 0 -> 1 -> 2
        // |  /
        // 3 -
        let amount_nodes = 4;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 10, 0));
        edges.push(Way::test(0, 3, 1, 1));
        edges.push(Way::test(1, 2, 1, 2));
        edges.push(Way::test(3, 1, 1, 3));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let (shortcuts, _used_edges) = calc_shortcuts(
            1,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &mut dijkstra,
        );

        // no need for a shortcut 0->1->2, because there is already the shortcut 3->1->2
        let expected_shortcuts = vec![Way::shortcut(3, 2, 2, 3, 2)];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn contract_simple_node() {
        // 0 -> 1 -> 2
        // |  /   \  |
        // 3 --->--- 4
        let amount_nodes = 5;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 1, 0));
        edges.push(Way::test(1, 2, 1, 2));
        edges.push(Way::test(0, 3, 1, 1));
        edges.push(Way::test(3, 1, 5, 4));
        edges.push(Way::test(1, 4, 4, 3));
        edges.push(Way::test(3, 4, 3, 5));
        edges.push(Way::test(4, 2, 1, 6));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let (shortcuts, _used_edges) = calc_shortcuts(
            1,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &mut dijkstra,
        );

        // there should be a shortcut 0->2, but no shortcuts 0->4, 3->2
        let expected_shortcuts = vec![Way::shortcut(0, 2, 2, 0, 2)];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn contract_disconnect() {
        // TODO check how edges are moved to new graph
    }
}
