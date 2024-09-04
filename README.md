# synerlogikos
A high-level, generic, abstracted Rust library for building any integration. It is designed to abstract away common patterns found in all integrations multiple times. For example, each object in any integration needs to, for each record, create, update, delete etc. This same pattern can be found for each record, in each direction.

If we do some quick maths, a bi-directional integration, with 10 records, which each need create would equate to repeating the same high-level pattern: 2 * 10 => 20 times.

Built by someone who has made many production integrations from scratch. My attempt is to reduce 100s (if not 1000s) of lines of code in each integration. Hopefully it could do the same for you.
