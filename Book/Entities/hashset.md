# The Hashset method

Hellooo I see you chose the hashset method :P

The code for this speaks for itself so I'll just show it-

```rust, noplaypen
use std::collections::HashSet;
struct HashsetMethod(HashSet<Entity>);

impl HashsetMethod {
    fn despawn(&mut self, entity: Entity) {
        self.0.insert(entity);
    }

    fn is_alive(&self, entity: Entity) -> bool {
        self.0.contains(&entity) == false
    }
}
```

Right there we go that was fairly easy we're done now right? :)  
no... :( 

```rust
# #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#pub struct Entity(u64);
#
#use std::collections::HashSet;
#struct HashsetMethod(HashSet<Entity>);
#
#impl HashsetMethod {
#    fn despawn(&mut self, entity: Entity) {
#        self.0.insert(entity);
#    }
#
#    fn is_alive(&self, entity: Entity) -> bool {
#        self.0.contains(&entity) == false
#    }
#}

fn main() {
    let mut statuses = HashsetMethod(HashSet::new());

    let e1_1 = {
        // Spawn an entity from one generator
        # Entity(0) // haha ur so sneaky hi 
    };
    statuses.despawn(e1_1);
    let e1_2 = {
        // Spawn the same entity from a second generator
        # Entity(0) // haha ur so sneaky hi
    };
    assert!(statuses.is_alive(e1_2));
}
```

Sooooooooo this is pretty problematic, we can spawn entities with other generators, despawn them, and then spawn a dead entity. Looking back over our code with this in mind we can also see that we can ask if an entity that hasnt been spawned yet is alive and it would say yes. So how should we handle this? We can use ``next_id`` to check if the entity passed in has been spawned yet. 

```rust
# #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#pub struct Entity(u64);
#
#use std::collections::HashSet;
#struct HashsetMethod(HashSet<Entity>);
#
impl HashsetMethod {
    fn despawn(&mut self, next_id: u64, entity: Entity) {
        // New
        if self.is_alive(next_id, entity) == false {
            return;
        }

        self.0.insert(entity);
    }

    fn is_alive(&self, next_id: u64, entity: Entity) -> bool {
        // New
        if entity.0 >= next_id {
            panic!("Attempted to use an entity from a different EntityGenerator");
        }

        self.0.contains(&entity) == false
    }
}

fn main() {
    let mut statuses = HashsetMethod(HashSet::new());

    let e1_1 = {
        // Spawn an entity from one generator
        # Entity(0) // haha ur so sneaky hi 
    };
    statuses.despawn(0, e1_1);
    let e1_2 = {
        // Spawn the same entity from a second generator
        # Entity(0) // haha ur so sneaky hi
    };
    assert!(statuses.is_alive(1, e1_2));
}
```

You'll notice that in ``is_alive`` we panic if the entity hasn't been spawned yet since it's definitely a bug on the user's side to mixup entities like this. You could also ``return false;`` here if you're opposed to panic'ing :)