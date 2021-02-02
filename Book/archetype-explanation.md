# What are archetypes?

Great question, and very necessary to understand before we write an archetype based ECS lol

Note: for this section when you read something like "an entity ``[T, T1, T2]``" it means that the entity has components of type ``T1``, ``T2`` and ``T3`` and \*nothing else*, if there's no square brackets it means the entity has those components and maybe more

One of the things we \*really* want out of our ECS is being able to iterate really fast since that's the primary way of using it. This is where archetypes come in, an archetype is a pool of entities that all have the same set of components, e.g. all entities with ``[T1, T2]`` will be stored in one archetype and all entities with ``[T1, T3]`` in another

What this lets us do is that an archetype that stores components ``[T, T1]`` can (in psuedocode) look something like:
```rust
struct Archetype {
    t_components: Vec<T>,
    t1_components: Vec<T1>,
}
```
Both of the ``Vec``s will be the same length since we \*only* store entities with ``[T, T1]`` components in this archetype. We also make sure the that the components at each index in the ``Vec``s correspond to the same entity. e.g.

``t_components[0]`` and ``t1_components[0]`` would be the components for the first entity stored in this archetype,
``t_components[1]`` and ``t1_components[1]`` for the second entitiy stored in this archetype etc

What this means is that when we want to iterate all entities that have ``T`` and ``T1`` components we can find this archetype (and other archetypes that have a ``Vec<T>`` and ``Vec<T1>``) and then return all the components we are interested in by just blindly iterating both ``Vec``s. This is super fast because all our components are all lined up in contiguous blocks and we never have to skip any which minimises branching and allows for good cache usage

To expand on the "and other archetypes" comment a bit, an entity with components ``[T, T1, T2]`` would \*not* have its components stored in this archetype, its component would be stored in something like this (psuedocode)
```rust
struct OtherArchetype {
    t_components: Vec<T>,
    t1_components: Vec<T1>,
    t2_components: Vec<T2>,
} 
```

This makes the general overview for the implementation of query's in an archetype ecs look something like:

  - Try find an archetype that stores the components we are interested in
  - If we can find one:
    - Create iterators over the relevent component ``Vec``s 
    - Call next() repeatedly on said iterators until they are finished
    - Go back to step #1
  - Else:
    - Our query has finished :)

So far archetypes sound pretty good performance wise right? well they have a big downside which is that adding and removing components from entities is \*\*slow\*\*, like \*really* slow. If we take the previous example of an entity with components ``[T, T1, T2]`` if we wanted to remove component ``T2`` and keep our amazing iteration performance we need to remove all three components from the ``[T, T1, T2]`` archetype and and then copy ``T`` and ``T1`` into the ``[T, T1]`` archetype. This is way expensive, and is why you will often see people \*heavily* cautioning against performing component add/removes with archetype ECS', bevy (a rust game engine) is even going as far as to [add an alternative component storage](https://www.google.com) in addition to archetypes just to allow for opt-in faster add/removes for specified component types.  