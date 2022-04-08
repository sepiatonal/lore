use std::collections::HashMap;
use std::mem;

use crate::AST;

struct VM {
    listeners: HashMap<u64, Listener>, // indexed by event signature
    behaviors: Vec<Behavior>, 
    entities: EntityWorld,
}

struct EntityWorld {
    archetypes: Vec<Archetype>,
    entity_lists: Vec<Vec<Entity>>,
}

impl EntityWorld {
    fn forall(behaviors_bitmask: u64, code: fn(Entity) -> ()) {

    }
}

struct Archetype {
    behaviors_bitmask: u64,
    entities_list_index: usize,
}

struct Entity {

}

struct Behavior {

}

struct Listener {
    
}

impl VM {
    pub fn create(ast: AST) {
        // TODO
    }
    
    pub fn run(&mut self) {

    }
}