#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Entity {
    pub(crate) index: u32,
    // GUIDE: trade offs of u16 vs u32 generation, why do we need a generation
    generation: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum EntityStatus {
    Alive,
    Dead,
    Tombstone,
}

pub(crate) struct EntityGenerator {
    entities: Vec<(u32, EntityStatus)>,
    despawned: Vec<u32>, // Indices into entities vec
}

impl EntityGenerator {
    pub(crate) fn new() -> Self {
        Self {
            // GUIDE: We could do with_capacity here if we wanted
            entities: Vec::new(),
            despawned: Vec::new(),
        }
    }

    pub(crate) fn is_alive(&self, entity: Entity) -> bool {
        // GUIDE: talk about moment where the generation field was unused and it made me realise there was a bug
        if let Some(&(gen, status)) = self.entities.get(entity.index as usize) {
            if gen == entity.generation && status == EntityStatus::Alive {
                return true;
            }
        }
        false
    }

    pub(crate) fn spawn(&mut self) -> Entity {
        if let Some(despawned_idx) = self.despawned.pop() {
            // GUIDE: talk about correctness of this case and the implicit assumption of usize > u32 throughout this module
            let (generation, status) = &mut self.entities[despawned_idx as usize];
            assert_eq!(*status, EntityStatus::Dead);
            assert!(*generation != u32::MAX); // GUIDE: Explain tombstones and why wrapping generation could be problematic

            *generation = *generation + 1; // We use regular + addition here instead of wrapping or saturating etc because we checked for != u32::MAX
            *status = EntityStatus::Alive;

            return Entity {
                index: despawned_idx as u32, // This cast wont lead to issues because we check to never spawn more than u32::MAX entities
                generation: *generation,
            };
        }

        // We only use a u32 for the index portion of entity's so spawning more than that is impossible
        if self.entities.len() == u32::MAX as usize {
            panic!("Too many entities spawned in world");
        }

        self.entities.push((0, EntityStatus::Alive));
        Entity {
            index: self.entities.len() as u32 - 1,
            generation: 0,
        }
    }

    pub(crate) fn despawn(&mut self, entity: Entity) -> bool {
        // GUIDE: this code originally used [] indexing because I assumed it was okay but forgot we could get entities from other worlds
        // I discovered this when writing tests
        let (gen, status) = match self.entities.get_mut(entity.index as usize) {
            Some(d) => d,
            None => return false,
        };

        // We could hard error when despawning an already despawned entity except that this would cause
        // users to have to wrap every call to this fn in a call to is_alive which would be pretty unergonomic
        if *status != EntityStatus::Alive {
            return false;
        }

        match *gen == u32::MAX {
            true => *status = EntityStatus::Tombstone,
            false => {
                *status = EntityStatus::Dead;
                self.despawned.push(entity.index)
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::EntityGenerator;
    use super::{Entity, EntityStatus};

    #[test]
    fn spawn_one() {
        let mut generator = EntityGenerator::new();
        let e1 = generator.spawn();

        assert!(generator.is_alive(e1));

        generator.despawn(e1);
        assert!(generator.is_alive(e1) == false);
    }

    #[test]
    fn other_world_entity() {
        let mut generator_1 = EntityGenerator::new();
        let mut generator_2 = EntityGenerator::new();

        let e1_1 = generator_1.spawn();
        assert!(generator_2.is_alive(e1_1) == false);
        assert!(generator_2.despawn(e1_1) == false);

        // GUIDE: maybe we should test that despawn call didnt mess with the next spawned entity
        let e1_2 = generator_2.spawn();
        assert_eq!(e1_1, e1_2);
    }

    #[test]
    fn generation_reuse() {
        let mut generator_1 = EntityGenerator::new();

        let e1 = generator_1.spawn();
        generator_1.despawn(e1);
        let e2 = generator_1.spawn();

        assert_eq!(
            e2,
            Entity {
                index: 0,
                generation: 1,
            }
        );
    }

    #[test]
    fn tombstone() {
        let mut generator_1 = EntityGenerator {
            entities: vec![(u32::MAX, EntityStatus::Alive)],
            despawned: Vec::new(),
        };

        let e1 = Entity {
            index: 0,
            generation: u32::MAX,
        };

        // GUIDE: lets just be really sure we manually created the entity correctly
        assert!(generator_1.is_alive(e1));

        generator_1.despawn(e1);
        // GUIDE: probably dont need to but might aswell check that tombstone entities are considered dead
        assert!(generator_1.is_alive(e1) == false);
        assert_eq!(generator_1.entities[0], (u32::MAX, EntityStatus::Tombstone));

        let e2 = generator_1.spawn();
        assert!(generator_1.is_alive(e2));
        assert_eq!(
            e2,
            Entity {
                index: 1,
                generation: 0,
            }
        );
    }

    #[test]
    fn double_despawn() {
        let mut generator_1 = EntityGenerator::new();
        let e1 = generator_1.spawn();
        assert!(generator_1.despawn(e1) == true);
        assert!(generator_1.despawn(e1) == false);

        // GUIDE: despawning pushes an entity to the Despawned vec... maybe we should check theres only one entry after calling that twice
        assert!(generator_1.despawned.len() == 1);
    }

    // GUIDE: if only we could test having more than u32::MAX entities would panic alas we would run out of ram
}
