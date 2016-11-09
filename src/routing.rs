extern crate online_dag;
use self::online_dag::rcdag::RcDag;

pub struct RouteEdge {
    // 0 corresponds to the source,
    // 1 corresponds to the delay-by-zero weight,
    // 2 corresponds to the delay-by-one weight, etc.
    slot_idx: u32,
}

pub type RouteNode=();

pub type RouteTree=RcDag<RouteNode, RouteEdge>;
