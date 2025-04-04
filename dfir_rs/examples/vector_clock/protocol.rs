use lattices::Max;
use lattices::map_union::MapUnionHashMap;
use serde::{Deserialize, Serialize};

pub type VecClock = MapUnionHashMap<String, Max<usize>>;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize, Debug)]
pub struct EchoMsg {
    pub payload: String,
    pub vc: VecClock,
}
