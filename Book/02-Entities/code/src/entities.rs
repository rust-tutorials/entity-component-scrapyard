#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity(u64);

pub(crate) struct EntityGenerator {
    next_id: u64,
    dead_entities: std::collections::HashSet<Entity>,
}

impl EntityGenerator {
    pub(crate) fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        if self.next_id == u64::MAX {
            panic!("Attempted to spawn an entity after running out of IDs");
        }
        self.next_id += 1;
        entity
    }

    pub(crate) fn despawn(&mut self, entity: Entity) {
        if self.is_alive(entity) {
            self.dead_entities.insert(entity);
        }
    }

    pub(crate) fn is_alive(&self, entity: Entity) -> bool {
        // self.next_id is the ID for the next entity so any entities with
        // that ID or higher haven't been spawned yet
        if entity.0 >= self.next_id {
            panic!("Attempted to use an entity in an EntityGenerator that it was not spawned with");
        }
        self.dead_entities.contains(&entity) == false
    }
}
