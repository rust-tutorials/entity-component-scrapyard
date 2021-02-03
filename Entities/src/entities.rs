#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity(u64);

struct EntityGenerator {
    next_id: u64,
    dead_entities: std::collections::HashSet<Entity>,
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
        if entity.0 < self.next_id {
            self.dead_entities.insert(entity);
        }
    }

    fn is_alive(&self, entity: Entity) -> bool {
        if entity.0 < self.next_id {
            return self.dead_entities.contains(&entity) == false;
        }
        false
    }
}
