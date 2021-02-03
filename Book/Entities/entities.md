# Entities

Lets start with implementing a basic struct that will let us spawn and despawn entities. Every ECS has something like this and hopefully we wont have to touch the more complicated stuff like query's or archetypes when we do this...

An entity in an ECS doesn't store any data it's effectively just a handle to some data stored in the ECS. A simple way to implement this would be to create a tuple struct that wraps a u32 or u64 or whatever size integer you want e.g.
```rust, noplaypen
pub struct Entity(u64);
```
we could then add some derives
```rust, noplaypen 
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity(u64);
```
This would work pretty well, entities would be cheap to copy, compare with eachother, and they're relatively small in size which is good because we'll have a \*lot* of entities in our ECS. Sequential ids would make for the code for generating new ids not super complex, we just add 1 to a counter. The first entity we spawn is ``Entity(0)`` the second is ``Entity(1)`` the third ``Entity(2)`` etc etc etc

Well, lets try write up this entity spawning code and see if it ends up with any suprise complexity

Lets start by making a struct with that counter and our spawn method that increments it
```rust, noplaypen
struct EntityGenerator {
    next_id: u64,
}

impl EntityGenerator {
    fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        self.next_id += 1;
        entity
    }
}
```

Okay well thats not a lot of code although how exactly does ``self.next_id += 1`` handle the case where ``self.next_id == u64::MAX`` we're unlikely to ever have this many entities but we probably ought to atleast see what will happen...

(click the run button :3)
```rust
# #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#pub struct Entity(u64);
#
#struct EntityGenerator {
#    next_id: u64,
#}
#
#impl EntityGenerator {
#    fn spawn(&mut self) -> Entity {
#        let entity = Entity(self.next_id);
#        self.next_id += 1;
#        entity
#    }
#}
#
let mut entity_generator = EntityGenerator { next_id: u64::MAX };
let entity = entity_generator.spawn();
dbg!(entity);
```

Oh well that's not great...
```
   Compiling playground v0.0.1 (/playground)
    Finished dev [unoptimized + debuginfo] target(s) in 1.44s
     Running `target/debug/playground`
thread 'main' panicked at 'attempt to add with overflow', src/main.rs:14:8
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

I know! We could explicitly use ``wrapping_add`` to avoid this panic.

oh but then two calls to ``EntityGenerator::spawn`` could return the same entity and that's definitely undesirable, maybe panic'ing is what we want here? For now lets handle this explicitly rather than relying on rust to insert overflow checks.

```rust
# #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#pub struct Entity(u64);
#
#struct EntityGenerator {
#    next_id: u64,
#}
#
#
impl EntityGenerator {
    fn spawn(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        if self.next_id == u64::MAX {
            panic!("Attempted to spawn an entity after running out of IDs");
        }
        self.next_id += 1;
        entity
    }
}

fn main() {
    let mut entity_generator = EntityGenerator { next_id: u64::MAX };
    let entity = entity_generator.spawn();
    dbg!(entity);
}
```

If we run this hopefully we panic with our new error message... 
```
thread 'main' panicked at 'Attempted to spawn an entity after running out of IDs'
```
Yep!

Okay the next thing we want to do is despawn entities, thats a pretty useful thing we're going to need for our ECS... We'll want a ``despawn`` function but what is this function even going to do, we cant store the dead/alive status on the Entity itself because we can copy an entity before its despawned and then it would still think it's alive...

I guess we need to store all the despawned entities in our ``EntityGenerator`` somewhere... a ``Vec<Entity>`` would be slow to check if an entity is dead so I guess we'll use a ``HashMap``

```rust, noplaypen, diff
struct EntityGenerator {
    next_id: u64,
    dead_entities: std::collections::HashMap<Entity, ???>,
}
```
Oh I guess there's nothing we need to store for the key so lets just use a ``HashSet`` instead
```rust, noplaypen
struct EntityGenerator {
    next_id: u64,
    dead_entities: std::collections::HashSet<Entity>,
}
```
Okay now lets just hookup this ``HashSet`` to the ``is_alive`` and ``despawn`` methods...
```rust
# #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#pub struct Entity(u64);
#
#struct EntityGenerator {
#    next_id: u64,
#    dead_entities: std::collections::HashSet<Entity>,
#}
#
impl EntityGenerator {
#    fn spawn(&mut self) -> Entity {
#        let entity = Entity(self.next_id);
#        if self.next_id == u64::MAX {
#            panic!("Attempted to spawn an entity after running out of IDs");
#        }
#        self.next_id += 1;
#        entity
#    }
#
#
    fn despawn(&mut self, entity: Entity) {
        self.dead_entities.insert(entity);
    }

    fn is_alive(&self, entity: Entity) -> bool {
        self.dead_entities.contains(&entity) == false
    }
}
```

Right there we go that was fairly easy we're done now right? :)

no... :( 

```rust
# #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#pub struct Entity(u64);
#
#struct EntityGenerator {
#    next_id: u64,
#    dead_entities: std::collections::HashSet<Entity>,
#}
#
#impl EntityGenerator {
#    fn spawn(&mut self) -> Entity {
#        let entity = Entity(self.next_id);
#        if self.next_id == u64::MAX {
#            panic!("Attempted to spawn an entity after running out of IDs");
#        }
#        self.next_id += 1;
#        entity
#    }
#
#
#    fn despawn(&mut self, entity: Entity) {
#        self.dead_entities.insert(entity);
#    }
#
#    fn is_alive(&self, entity: Entity) -> bool {
#        self.dead_entities.contains(&entity) == false
#    }
#}
fn main() {
    let mut generator_1 = EntityGenerator { next_id: 0, dead_entities: std::collections::HashSet::new() };
    let mut generator_2 = EntityGenerator { next_id: 0, dead_entities: std::collections::HashSet::new() };

    let e1_1 = generator_1.spawn();
    generator_2.despawn(e1_1);
    let e1_2 = generator_2.spawn();
    assert!(generator_2.is_alive(e1_2));
}
```

Sooooooooo this is pretty problematic, we can spawn entities with other generators, despawn them, and then spawn a dead entity. Looking back over our code with this in mind we can also see that we can ask if an entity from another generator is alive and it would say yes. Lets fix that quickly by just checking if the provided entity's index is < the highest ID we handed out..

```rust
# #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#pub struct Entity(u64);
#
#struct EntityGenerator {
#    next_id: u64,
#    dead_entities: std::collections::HashSet<Entity>,
#}
#
impl EntityGenerator {
#     fn spawn(&mut self) -> Entity {
#         let entity = Entity(self.next_id);
#         if self.next_id == u64::MAX {
#             panic!("Attempted to spawn an entity after running out of IDs");
#         }
#         self.next_id += 1;
#         entity
#     }
#
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

fn main() {
    let mut generator_1 = EntityGenerator { next_id: 0, dead_entities: std::collections::HashSet::new() };
    let mut generator_2 = EntityGenerator { next_id: 0, dead_entities: std::collections::HashSet::new() };

    let e1_1 = generator_1.spawn();
    assert!(generator_2.is_alive(e1_1) == false);
    generator_2.despawn(e1_1);
    let e1_2 = generator_2.spawn();
    assert!(generator_2.is_alive(e1_2));
}
```

And there we go both of those problems are solved now... We should probably write some tests to see if this all works huh