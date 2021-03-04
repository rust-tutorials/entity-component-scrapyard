# Creating our struct

Before we get started, in our `src/lib.rs` file we should add `mod archetype;` and then make a 
file at `src/archetype.rs`. All of the code in this chapter will be inside of this `archetype.rs` file

Our archetype struct needs to be able to store any number of component columns. The
way to do this normally would be a `Vec<ComponentColumn>` however our component columns
are going to be Vec's which are generic over the type they store. This is a bit problematic
for since we want to store a set of things that aren't the same type

Luckily rust has a way of yeeting type information away- trait objects. We can implement a trait
for every type of Vec and then store a `Vec<Box<dyn TypeErasedVec>>`. We can then downcase the 
trait object back to a concrete type whenever we need to. e.g. when we add/remove components

```rust
use std::any::Any;

trait ComponentColumn: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> ComponentColumn for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct Archetype {
    entities: Vec<crate::Entity>,
    columns: Vec<Box<dyn ComponentColumn>>,
}
```

Here we define our `ComponentColumn` trait to be a supertrait of the `Any` trait. You might not
have come across the `Any` trait before, it doesn't tend to come up very often when writing rust.
The general idea behind the `Any` trait is that we cast a type to a trait object and then call
`downcast_ref/mut` with a generic to turn it back into a concrete type. 
You can see the docs for std::any::Any [here](https://doc.rust-lang.org/std/any/trait.Any.html)

## Methods for creating an instance of our Archetype struct

We need some methods for creating instances of our `Archetype` struct. There are two situations that
we'll be needing to create an `Archetype` in:
- When we add/remove a component we'll have to create the archetype that we need to move the entity to
- When we spawn an entity we'll have to create an archetype matching the component's given

We'll start with the add/remove situation because that's going to be easier to write.

The first thing we want to do is make a method on our `ComponentColumn` trait that returns a 
`Box<dyn ComponentColumn>` of the same type of Vec. This will let us create an archetype with the same
set of columns as an existing one. 

```rust
trait ComponentColumn: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn new_empty_column(&self) -> Box<dyn ComponentColumn>;
}

impl<T: 'static> ComponentColumn for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn new_empty_column(&self) -> Box<dyn ComponentColumn> {
        Box::new(Vec::<T>::new())
    }
}
```

Now lets look at the method for creating the archetype we want to move our entity to after an add/remove op: 

```rust
impl Archetype {
    fn new_from_add<T: 'static>(from_archetype: &Archetype) -> Archetype {
        let mut columns: Vec<_> = from_archetype
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        todo!("Actually add the column");

        Archetype {
            entities: Vec::new(),
            columns,
        }
    }

    fn new_from_remove<T: 'static>(from_archetype: &Archetype) -> Archetype {
        let mut columns: Vec<_> = from_archetype
            .columns
            .iter()
            .map(|column| column.new_empty_column())
            .collect();

        todo!("Actually remove the column");

        Archetype {
            entities: Vec::new(),
            columns,
        }
    }
}
```

We could \*technically\* merge these into one method and choose whether to add or remove
depending on whether `Vec<T>` is already present in the columns. However whenever we call
this method we'll \*know\* whether it's meant to be an add/remove, so if we make them separate
methods we can add some checks that it's valid to add/remove a component from the archetype.

The code for `new_from_add`:

```rust
impl Archetype {
    fn new_from_add<T: 'static>(from_archetype: &Archetype) -> Archetype {
#        let mut columns: Vec<_> = from_archetype
#            .columns
#            .iter()
#            .map(|column| column.new_empty_column())
#            .collect();
        /* snip */

        assert!(columns
            .iter()
            .position(|column| column.as_any().is::<Vec<T>>())
            .is_none());
        columns.push(Box::new(Vec::<T>::new()));

        /* snip */
#        Archetype {
#            entities: Vec::new(),
#            columns,
#        }
    }
}
```

We just `assert!` here instead of trying to recover because it's going to be a bug in our ECS if 
this assert fires and we really want that to happen loudly.

The code for `new_from_remove`:

```rust
    fn new_from_remove<T: 'static>(from_archetype: &Archetype) -> Archetype {
#        let mut columns: Vec<_> = from_archetype
#            .columns
#            .iter()
#            .map(|column| column.new_empty_column())
#            .collect();
        /* snip */

        let idx = columns
            .iter()
            .position(|column| column.as_any().is::<Vec<T>>()).unwrap();
        columns.remove(idx);

        /* snip */
#        Archetype {
#            entities: Vec::new(),
#            columns,
#        }
    }
```

Same reason as above for why we just panic in here rather than try to recover :)

That should be everything we need for these two functions but before we
move on lets add some tests to verify everything is working correctly

## Tests for add/remove constructors

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /* todo tests */
}
```

First tests we should add are some simple ones for testing those asserts/unwraps 
fire when trying to call `new_from_(add/remove)` on incorrect archetypes.

```rust, noplaypen
{{#rustdoc_include ./code/src/archetype.rs:simple_tests}}
```

The next test we want is just something to call `new_from_(add/remove)` a bunch and assert
that all the columns that we expect to be present are actually present:

```rust, noplaypen
{{#rustdoc_include ./code/src/archetype.rs:thicc_test}}
```

## Archetype constructor for when spawning entities

Now that we've finished up the constructors for when we add/remove components on an entity we
need to make one for when we spawn an entity and need to build an archetype for it.

This one is a bit tricker because we want to take any # of component types when we create an
archetype to spawn our entity into. We'll want some kind of builder struct that we can repeatedly
call a function on to add component columns.

Something like this:

```rust
struct ColumnsBuilder(Vec<Box<dyn ComponentColumn>>);

impl ColumnsBuilder {
    fn with_column_type<T: 'static>(mut self) -> Self {
        if let Some(_) = self
            .0
            .iter()
            .filter(|col| col.as_any().type_id() == std::any::TypeId::of::<Vec<T>>())
            .next()
        {
            panic!("Attempted to create invalid archetype");
        }

        self.0.push(Box::new(Vec::<T>::new()));
        self
    }
}
```

We have to check that we don't try to create an archetype with two columns for the same component
type. i.e. an archetype for entities with components `[T, T]` is nonsensical as an entity cannot
have the same component added twice. Currently I just panic here if we detect that but it would 
be possible to return a `Result` here and propagate it up to the user

Now that we have a `ColumnsBuilder` lets add some methods to `Archetype` to use it

```rust
impl Archetype {
    /* snip */

    fn builder() -> ColumnsBuilder {
        ColumnsBuilder(Vec::new())
    }

    fn new_from_columns(columns: ColumnsBuilder) -> Archetype {
        Archetype {
            entities: Vec::new(),
            columns: columns.0,
        }
    }
}
```

An alternative way of implementing this would be to implement a trait for tuples of length 1 to 
some arbitrary limit and then create the columns in that trait. This has the downside of needing
to use macros for the trait impl and we also would have a limit on how many components could be
spawned on an entity without adding components separately. (We'll doing something like this later
on when we implement iterators over our archetypes)

This should be us done with the method we'll use when creating an `Archetype` to spawn an entity
into, we just need to add some tests before moving on :)

## Tests for ColumnsBuilder

We'll want a test that our duplicate column checks work and also a general test that we have the
expected columns after building an archetype from the `ColumnsBuilder`:

```rust
#[cfg(test)]
mod tests {
    /* snip */

{{#include ./code/src/archetype.rs:columns_builder_tests}}
}
```

The full source code for this chapter can be viewed [here](https://github.com/rust-tutorials/entity-component-scrapyard/tree/main/Book/03-Archetypes/code)