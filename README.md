# Remnant

Remnant is a distributed shared timeline service. It comes in several
different modules. This module is responsible for storing events and
querying against stored events.  Other modules are responsible for
transmitting events to different nodes across different mediums such
as TCP or UDP.

# Write Operations

All write operations take some number of parameters as inputs and
result in a "compiled" output object and it's hash. The hash is always
embedded into the compiled object.

## Create

The `create` operation creates a new named timeline. It has no
ancestors. The result of a create operation has a user-defined name
with a maximum length and a universally unique identifier (UUID).

## Append

The `append` operation adds a new event to a timeline. It has a single
ancestor. This operation also accepts a payload that will be compiled
into the event object.

## Join

The `join` operation adds a special event to a timeline. It has two
ancestors. This operation creates a point where two events and their
ancestors merge back into the same timeline.
