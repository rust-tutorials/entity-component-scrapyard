# What are archetypes?

Archetypes are the underlying storage method we will be using to store components in our ECS. In 
our ECS we will use something like ``Vec<Archetype>`` to store everything.  
An archetype stores \*all* of the components on an entity and also only stores components from entities 
with the same set of components. E.G. an entity with \*only* ``[T1, T2, T3]`` components would
have all components stored in one archetype, and an entity with \*only* ``[T1, T2, T3, T4]`` 
components would have all the components stored in a different archetype.

Let's say we have an archetype, we'll call it ``A_123``. ``A_123`` is storing 
components for entities that only have components ``[T1, T2, T3]``.  
When we spawn an entity like so:
```rust, noplaypen
world.spawn((
  T1 { .. },
  T2 { .. },
  T3 { .. },
));
```
We need to find an archetype to place all these components into. As mentioned previously we want to find if there's
an archetype that only stores components for entities with ``[T1, T2, T3]``, and, luckily for us, there is! It's ``A_123``.

Let's take a quick look at what ``A_123`` might look like internally and how we would go about placing these components into it.  
In psuedocode we have some archetype struct that looks a bit like:
```rust, noplaypen
struct A_123 {
  column_1: Vec<T1>,
  column_2: Vec<T2>,
  column_3: Vec<T3>,
}
```

You might be wondering why we store each component type in a separate vec rather than something like this:
```rust, noplaypen
struct A_123(Vec<(T1, T2, T3)>);
```
There are a few reasons for this,
  - The borrow checker will get angry at us if we have two Query's where one wants to access components ``T1`` mutably and
  the other wants to access ``T2`` immutably (We cant create an iterator over only \*parts* of the (T1, T2, T3) tuple afterall)
  - It's more performant to store the components in separate Vecs as when we iterate only ``T1`` we dont needlessly load 
  the ``T2`` and ``T3`` components into the cpu's cache.

With that explained lets look at some psuedocode for that ``spawn`` function we used earlier.  
```rust, noplaypen
fn spawn(&mut self, data: (T1, T2, T3)) -> Entity {
  let a_123 = /* magically get the A_123 archetype */;

  let entity = self.entity_generator.spawn();
  a_123.entities.push(entity);

  a_123.column_1.push(data.0);
  a_123.column_2.push(data.1);
  a_123.column_3.push(data.2);

  entity
}
```
...Huh interesting how all this psuedocode looks awfully like Rust... I'm sure its just a coincedence ;)  

There are a few things to talk about the above code:
  - What's this ``entities`` field that suddenly appeared?
  - Do we need to store any kind of metadata so that we know which entity every element
  in each of the component vecs corresponds to?

Both of these things are \*actually* one and the same.  
Because we store entities with \*only* components ``[T1, T2, T3]`` in this archetype, the component 
vecs will always be the same length. This means that when we push components to the front of the vecs 
they'll all end up at the same index.  

What we can do with this knowledge is have have a ``Vec<Entity>`` in the archetype and 
push the spawned ``Entity`` to the entities vec. We then have an implicit mapping where for every index in 
the component columns we can access the ``entities`` vec at the same index to check what entity this component is for. 
It's for the same reason that we can also access the other component columns at this index and the component will be for
the same entity.

That last point is why Query's in archetype based ECS' are \*so* fast. When we want to iterate all ``T1`` and ``T2`` components
in our world we can just find every archetype that has a column for ``T1`` and also a column for ``T2`` and then blindly iterate
both Vecs at the same time. Some psuedocode to hopefully help demonstrate what I mean:
```rust, noplaypen 
fn iterate_T1_and_T2(archetype: &A_123) -> impl Iterator<Item = (&T1, &T2)> {
  archetype.column_1.iter().zip(archetype.column_2.iter())
}
```

Now that you (hopefully) have an understanding of how components get stored in archetype based ECS' and the performance
advantages we can get from it, it's time to talk about one of the biggest flaws of the archetype model. Adding/Removing components
is \*really\* slow.. like.. \*\*really\*\* slow.

Let's continue with our previous example of spawning our entity with components ``[T1, T2, T3]``. It's sitting pretty comfortably in
our ``A_123`` archetype and we'll have super fast iteration times if we add more entities to this archetype and query for their components.

Now lets try adding a component to our entity. As previously mentioned our ``A_123`` archetype stores entities which \*only* have 
``[T1, T2, T3]`` components as this lets us have amazing iteration speeds. Now let's say we add a component ``T4`` to out entity, 
we would no longer fit this criteria which means we cant store our entity's components in ``A_123``, we'll have to make a second archetype- 
``A_1234`` it will store components for entities who only have ``[T1, T2, T3, T4]``

Lets write some quick psuedocode and see if the performance problem here speaks for itself:

```rust, noplaypen
fn add_T4_to_entity_in_A_123(&mut self, entity: Entity, data: T4) {
  let a_123 = /* magically get the A_123 archetype */;
  let index = /* magically get the index in the 
  component columns corresponding to the entity */;

  let a_1234 = /* magically get the A_1234 archetype */;

  a_1234.entities.push(entity);
  a_1234.column_4.push(data);

  // Oh no this doesnt look cheap at all
  a_1234.column_1.push(a_123.column_1.remove(index));
  a_1234.column_2.push(a_123.column_2.remove(index));
  a_1234.column_3.push(a_123.column_3.remove(index));

  self.archetypes.push(a_1234);
}
```

Aaaaaaaaand yep... we have to move all of the components out of the columns in ``A_123``. We then have to 
push the removed component to our new archetype's columns. That's a lot of moving memory which isn't the 
fastest thing in the world- to say the least

There's not a whole lot we can do about this. The need to move all this memory around when adding/removing components is
entirely necessary for the previously mentioned amazing iteration performance. Archetype ECS' are inherently about trading in 
add/remove performance in exchange for iteration performance. 

There are others ways to model your ECS such as sparsesets which have signficantly better add/remove performance
in exchange for worse iteration performance relative to archetype based ECS' (still fast though :P). There's no one 
best way to model an ECS, sparseset and archetype ECS' just make different tradeoffs :)