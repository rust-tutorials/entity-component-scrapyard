# A basic generator

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

Oh, well that's not great...
```
   Compiling playground v0.0.1 (/playground)
    Finished dev [unoptimized + debuginfo] target(s) in 1.44s
     Running `target/debug/playground`
thread 'main' panicked at 'attempt to add with overflow', src/main.rs:14:8
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

We could explicitly use ``wrapping_add`` to avoid this panic, but then two calls to ``EntityGenerator::spawn`` could return the same entity and that's definitely undesirable. ``u64::MAX`` is a \*really* big number so whether we panic or use ``wrapping_add`` is unlikely to make much of a difference. For this guide I'm just going to panic but if you want to wrapping_add that's also fine :)
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

Okay the next thing we want to do is mark entities as dead, that's a pretty core thing for our ECS huh :P? We'll want a ``despawn`` function but how is this function even going to work? We cant store the dead/alive status on the Entity itself because we can copy an entity before it's despawned and then it would still think it's alive. We need to store all the despawned status of entities in our ``EntityGenerator``

There are a few options for this
  - A ``Vec<Entity>`` where we push dead entities to it, we would have to iterate the entire vec to check if an entity is dead which could be \*super* slow so this is a non-starter.
  - A ``Vec<bool>`` which we index with the entity's u64 and set to true when we despawn the entity. This would be super fast to check if an entity is dead/alive, if you're concerned about ram consumption with this you could use a bitset instead which would have 1/8th the usage of ``Vec<bool>``
  - A ``HashSet<Entity>`` (HashSets are effectively just ``HashMap<T, ()>``) and we just insert dead entities into it. This would also be pretty fast to check if an entity is dead/alive

Option #2 and #3 would both be valid choices but for this guide I'm going to go with option... BOTH, the guide is called entity-component-\*scrapyard* for a reason ;)

For the sake of this guide im going to write a little boilerplate to let me switch between both implementations but you wont need to do any of this, I'll just quickly show this and then move onto the individual chapters

```rust, noplaypen
enum EntityStatuses {
    // We'll create these structs in the individual chapters :)
    HashsetMethod(HashsetMethod),
    BitsetMethod(BitsetMethod),
}

impl EntityStatuses {
    fn despawn(&mut self, entity: Entity) {
        match self {
            Self::HashsetMethod(hashset) => hashset.despawn(entity),
            Self::BitsetMethod(bitset) => bitset.despawn(entity),
        }
    }

    fn is_alive(&self, next_id: u64, entity: Entity) -> bool {
        match self {
            Self::HashsetMethod(hashset) => hashset.is_alive(entity),
            Self::BitsetMethod(bitset) => bitset.is_alive(entity),
        }
    }
}
```

And now I'll just plop this enum into ``EntityGenerator`` and we can move onto the next chapter :)

```rust, noplaypen
struct EntityGenerator {
    next_id: u64,
    entity_statuses: EntityStatuses,
}

impl EntityGenerator {
#     fn spawn(&mut self) -> Entity {
#         let entity = Entity(self.next_id);
#         if self.next_id == u64::MAX {
#             panic!("Attempted to spawn an entity after running out of IDs");
#         }
#         self.next_id += 1;
#         entity
#     }
    // -Snip

    fn despawn(&mut self, entity: Entity) {
        self.entity_statuses.despawn(entity)
    }

    fn is_alive(&self, entity: Entity) -> bool {
        self.entity_statuses.is_alive(entity)
    }
}
```

The hashet method is [here](./hashset.md) and the bitset method is [here](./bitset.md)