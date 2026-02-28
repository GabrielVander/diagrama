# Diagrama

A basic diagram rendering tool implemented in Rust

## Support

We currently support only PlantUML

### PlantUML

We implement a basic PlantUML parser currently supporting:

- ❌ Classes
- ❌ Interfaces
- ❌ Enums
- ❌ Abstract classes
- ❌ Components
- ❌ Actors
- ❌ Databases
- ❌ Fields
- ❌ Methods
- ❌ Visibility
- ❌ Composition
- ❌ Aggregation
- ❌ Inheritance
- ❌ Dependency
- ❌ Association
- ❌ Packages / namespaces
- ❌ Notes
- ❌ Title
- ❌ Direction
- ❌ Generics
- ❌ Annotations
- ❌ Advanced arrow syntax
- ❌ Multiplicity
- ❌ Modifiers parsing
- ❌ Skinparams (ignored)

## Architecture

### Clean Architecture

Diagrama follows Uncle Bob's Clean Architecture. It uses the following layers:

```text
┌──────────────────────────────┐
│         Entity Layer         │
├──────────────────────────────┤
│        Use Case Layer        │
├──────────────────────────────┤
│        Adapters Layer        │
├──────────────────────────────┤
│     Infrastructure Layer     │
└──────────────────────────────┘
```

- Entity Layer
  Business objects of the application. They encapsulate the most general and
high-level rules. They are the least likely to change when something external changes

- Use Case Layer
  The software in this layer contains application specific business rules. It
encapsulates and implements all of the use cases of the system. These use cases
orchestrate the flow of data to and from the entities, and direct those entities
to use their enterprise wide business rules to achieve the goals of the use case
  - Gateways
    Interfaces defined by the use cases that allows it to communicate with
  outer layers

- Adapters Layer
  Interface Adapters. The software in this layer is a set of adapters that
convert data from the format most convenient for the use cases and entities, to
the format most convenient for some external agency. Also in this layer is any
other adapter necessary to convert data from some external form, such as an
external service, to the internal form used by the use cases and entities. This
is where Gateways implementations reside

- Infrastructure Layer
  Frameworks and Drivers. The outermost layer is generally composed of
frameworks and tools

### Vertical Slicing

In addition, Diagrama also makes use of vertical slices (feature-specific
crates orchestrated via cargo workspace), currently:

- lib-core
  - Contains the core of the application, typically it's where most entities
  and use cases definitions reside (Entity and Use Case Layers)
- lib-plant_uml
  - Contains PlantUML-specific code. It's where the use cases' gateways are
  implemented with PlantUML specifics (Adapters Layer). It's also where the
  language parsing logic is implemented (Infrastructure Layer)
