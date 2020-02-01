extern crate bincode;
extern crate osmpbfreader;
extern crate rayon;
extern crate serde;

mod constants;
mod contraction;
mod dijkstra;
mod grid;
mod helper;
mod min_heap;
mod offset;
mod ordering;
mod osm_parsing;
mod osm_pbf;
mod structs;
mod visited_list;

use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::time::Instant;

use crate::constants::*;
use crate::structs::*;

// Options: Car, Bicycle, Pedestrian
const TRAVEL_TYPE: TravelType = TravelType::Car;
// Options: Time, Distance
const OPTIMIZE_BY: OptimizeBy = OptimizeBy::Distance;

fn main() {
    let overall_time = Instant::now();

    let mut nodes = Vec::<Node>::new();
    let mut full_edges = Vec::<OsmWay>::new();
    let mut up_offset = Vec::<EdgeId>::new();
    let mut down_offset = Vec::<EdgeId>::new();
    let mut edges = Vec::<Way>::new();
    let mut grid_offset = Vec::<GridId>::new();
    let mut grid = Vec::<NodeId>::new();

    // storing mapping of own-ids and osm-ids
    let mut osm_id_mapping = HashMap::<i64, usize>::new();

    let filename = helper::get_filename();

    let pbf_time = Instant::now();
    let mut pbf = osm_pbf::get_pbf(&filename);
    // store all way-IDs that are having the "highway" tag. with speed-limit
    osm_pbf::read_edges(&mut pbf, &mut full_edges, &mut osm_id_mapping);
    let amount_nodes = osm_id_mapping.len();
    // store all geo-information about nodes
    osm_pbf::read_ways(&mut pbf, &mut nodes, &mut osm_id_mapping);
    println!("Reading PBF in: {:?}", pbf_time.elapsed());

    let weight_time = Instant::now();
    helper::calc_edge_distances(&mut full_edges, &nodes);
    helper::edges_to_weight(&mut edges, &full_edges);
    println!("Getting weights in: {:?}", weight_time.elapsed());

    // generate offset arrays
    let mut down_index =
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

    // contraction hierarchies
    let contraction_time = Instant::now();
    contraction::run_contraction(&mut nodes, &mut edges, &mut up_offset, &mut down_offset, &mut down_index);
    println!("Contraction in: {:?}", contraction_time.elapsed());

    // generate grid
    let grid_time = Instant::now();
    let grid_bounds = grid::generate_grid(&mut grid, &mut grid_offset, &nodes);
    println!("Generate grid in: {:?}", grid_time.elapsed());

    println!("#nodes: {:?}", nodes.len());
    println!("#edges: {:?}", edges.len());

    // combine everything
    let result = FmiFile {
        nodes,
        edges,
        up_offset,
        down_index,
        down_offset,
        grid,
        grid_offset,
        grid_bounds,
        optimized_by: OPTIMIZE_BY,
    };

    // save results to disk
    let output_file = helper::write_to_disk(filename, result);

    println!("Overall: {:?}", overall_time.elapsed());
    println!("Output is written to: {}", output_file);
}
