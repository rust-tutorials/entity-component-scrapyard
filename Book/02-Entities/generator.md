# A basic entity generator

Lets start with implementing a basic struct that will let us spawn and despawn entities. Every ECS has something like this and by starting here we can make progress without tackling some of the harder stuff immediately :)

Every ECS that has performance as a goal will not store data \*in* their Entity struct but 
will instead have the Entity be a handle to component data. This is because when we ask 
the ECS for all components ``T1`` and ``T2`` if they're stored in the Entity struct they'll 
never be in cache when we go to access them. (This also has many borrowck issues, maybe for another book though :>)
Whereas if we store a bunch of our ``T1`` components in a Vec, when we iterate through it 
we'll end up having the next components loaded into cache which is far more efficient.

A simple way to implement this "Entity as a handle" thing would be to create a tuple struct that wraps a u32 or u64 or whatever size integer you want e.g.
```rust, noplaypen
pub struct Entity(u64);
```
we could then add some derives
```rust, noplaypen 
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Entity(u64);
```
This would work pretty well, entities would be cheap to copy, compare with eachother, 
and they're relatively small in size which is good because we'll have a \*lot* of 
entities in our ECS. Sequential ids hopefully will also make the code for generating new 
ids not super complex, we just add 1 to a counter.  
The first entity we spawn is ``Entity(0)`` the second is ``Entity(1)`` the third ``Entity(2)`` etc etc etc

Well, lets try write up this entity spawning code and see if it ends up with any suprise complexity.
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

Okay well thats not a lot of code although how exactly does ``self.next_id += 1`` handle 
the case where ``self.next_id == u64::MAX`` we're unlikely to ever have this many 
entities but we probably ought to atleast see what will happen...

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

Oh, well that's not great...

We could explicitly use ``wrapping_add`` to avoid this panic, but then two calls to 
``EntityGenerator::spawn`` could return the same entity and that's definitely undesirable. 
``u64::MAX`` is a \*really* big number so whether we panic or use ``wrapping_add`` is unlikely to make much of a difference.   
For this guide I'm just going to panic but if you want to wrapping_add that's also fine :)  
Note: overflow only panics in debug mode so if we want to panic on overflow we need to explicitly check for it
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

Okay the next thing we want to do is mark entities as dead, that's a pretty core thing for our ECS huh :P?  
We'll want a ``despawn`` function but how is this function even going to work? We cant store the 
dead/alive status on the Entity itself because we can copy an entity before it's despawned and then 
it would still think it's alive.  
We need to store all the despawned status of entities in our ``EntityGenerator``

There are a few options for this
  - A ``Vec<Entity>`` where we push dead entities to it, we would have to iterate the entire vec to 
  check if an entity is dead which could be \*super* slow so this is a non-starter.
  - A ``Vec<bool>`` which we index with the entity's u64 and set to true when we despawn the entity. 
  This would be super fast to check if an entity is dead/alive, if you're concerned about 
  ram consumption with this you could use a bitset instead which would have 1/8th the usage of ``Vec<bool>``
  - A ``HashSet<Entity>`` and we just insert dead entities into it. This would also be pretty fast to check if an entity is dead/alive

Option #2 and #3 would both be valid choices but for this guide I'm going to go with the HashSet option because its simpler :) Option #2 should work well as a drop-in replacement for the hashset code though so feel free to do that on your own! :)

# The Hashset method

The code here will likely speak for itself so I'll just show it-

```rust, noplaypen
struct EntityGenerator {
    next_id: u64,
    // New
    dead_entities: std::collections::HashSet<Entity>,
}

impl EntityGenerator {
    // -Snip

    fn despawn(&mut self, entity: Entity) {
        self.dead_entities.insert(entity);
    }

    fn is_alive(&self, entity: Entity) -> bool {
        # I like my ``== false`` okay
        self.dead_entities.contains(&entity) == false
    }
}
```

At first glance this seems pretty correct right? but it isn't ^^" there's an implicit assumption in this code that 
we only ever receive entities that were spawned from this ``EntityGenerator`` but we can just ignore that and cause
the generator to spawn a dead entity like so:

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
#    fn despawn(&mut self, entity: Entity) {
#        self.dead_entities.insert(entity);
#    }
#
#    fn is_alive(&self, entity: Entity) -> bool {
#        self.dead_entities.contains(&entity) == false
#    }
#}
#
fn main() {
    use std::collections::HashSet;
    let mut gen_1 = EntityGenerator { next_id: 0, dead_entities: HashSet::new() };
    let mut gen_2 = EntityGenerator { next_id: 0, dead_entities: HashSet::new() };

    let e1_1 = gen_1.spawn();
    gen_2.despawn(e1_1);
    let e1_2 = gen_2.spawn();
    assert!(gen_2.is_alive(e1_2));
}
```

This is pretty problematic to say the least :P We can spawn entities with other generators, despawn them, and then spawn a dead entity. Looking back over our code with this in mind we can also see that if we call ``is_alive`` on an unspawned entity it would say yes... Soooooo how can we check for this?  
We could store another Hashet for every spawned entity but that would be really slow and waste memory, what's better is that we can compare ``self.next_id`` to the entity's ``u64`` to see if its indistinguishable from an entity spawned from this generator e.g.

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
    // -Snip

    fn despawn(&mut self, entity: Entity) {
        if self.is_alive(entity) {
            self.dead_entities.insert(entity);
        }
    }

    fn is_alive(&self, entity: Entity) -> bool {
        // self.next_id is the ID for the next entity so any entities with 
        // that ID or higher haven't been spawned yet
        if entity.0 >= self.next_id {
            panic!("Attempted to use an entity in an EntityGenerator that it was not spawned with");
        }
        self.dead_entities.contains(&entity) == false
    }
}

# #[allow(unreachable_code)]
fn main() {
    use std::collections::HashSet;
    let mut gen_1 = EntityGenerator { next_id: 0, dead_entities: HashSet::new() };
    let mut gen_2 = EntityGenerator { next_id: 0, dead_entities: HashSet::new() };

    let e1_1 = gen_1.spawn();
    assert!(gen_2.is_alive(e1_1) == false);
    gen_2.despawn(e1_1);
    let e1_2 = gen_2.spawn();
    assert!(gen_2.is_alive(e1_2) == unreachable!());
}
```

I decided to ``panic`` in the ``is_alive`` method rather than ``return false`` because its pretty safe to say that it would be unintentional to do this so we should fail loudly to bring attention to it. It would also be completely fine to ``return false`` if you're opposed to panic'ing in libraries unnecessarily :)

To wrap this chapter up lets move all this code into a separate module, reexport the entity struct and then set everything to pub(crate) since we'll likely need this all elsewhere.


```rust, noplaypen
// /src/entities.rs
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
```
```rust, noplaypen
// /src/lib.rs
#![forbid(unsafe_code)]
pub(crate) mod entities;
pub use entities::Entity;
```

The full source code for this chapter can be viewed [here](https://github.com/rust-tutorials/entity-component-scrapyard/tree/main/Book/02-Entities/code)

Now that we have entity spawning and despawning working it's about time to start storing some components, for that we need to learn about what archetypes are. Luckily that's exactly what the next chapter is for!