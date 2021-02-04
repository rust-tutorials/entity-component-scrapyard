#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity(u64);

enum EntityStatuses {
    HashsetMethod(HashsetMethod),
    BitsetMethod(BitsetMethod),
}

impl EntityStatuses {
    fn despawn(&mut self, next_id: u64, entity: Entity) {
        match self {
            Self::HashsetMethod(hashset) => hashset.despawn(next_id, entity),
            Self::BitsetMethod(bitset) => bitset.despawn(next_id, entity),
        }
    }

    fn is_alive(&self, next_id: u64, entity: Entity) -> bool {
        match self {
            Self::HashsetMethod(hashset) => hashset.is_alive(next_id, entity),
            Self::BitsetMethod(bitset) => bitset.is_alive(next_id, entity),
        }
    }
}

use std::collections::HashSet;
struct HashsetMethod(HashSet<Entity>);

impl HashsetMethod {
    fn despawn(&mut self, next_id: u64, entity: Entity) {
        if self.is_alive(next_id, entity) == false {
            return;
        }

        self.0.insert(entity);
    }

    fn is_alive(&self, next_id: u64, entity: Entity) -> bool {
        if entity.0 >= next_id {
            panic!("Attempted to use an entity from a different EntityGenerator");
        }

        self.0.contains(&entity) == false
    }
}

struct EntityGenerator {
    next_id: u64,
    entity_statuses: EntityStatuses,
}

impl EntityGenerator {
    fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        if self.next_id == u64::MAX {
            panic!("Attempted to spawn an entity after running out of IDs");
        }
        self.next_id += 1;
        entity
    }

    fn despawn(&mut self, entity: Entity) {
        self.entity_statuses.despawn(self.next_id, entity)
    }

    fn is_alive(&self, entity: Entity) -> bool {
        self.entity_statuses.is_alive(self.next_id, entity)
    }
}
