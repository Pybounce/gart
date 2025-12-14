## Language

- Allowing underscores in numbers (ie 1_000_000)
- Ranges
- String interpolation

## Next Up!

- [ ] Literal compiler support (null/true/false)
- [ ] String support
  - Which means working out objects...end me pls.
- [ ] Control flow
- [ ] Locals

**Static Typing**

- This matters now since it either may change how the compiler is structured completely OR it'll change how globals work. 2 Pass will be easier to type check over patching.
- _PROS_
  - Static typing is the cats pajamas
  - Leaving things to compile time is pretty much always better and I love it
- _CONS_
  - Added compiler complexity
    - The compiler is already single pass
    - This would more or less REQUIRE the globals to be done with 2 pass
  - REPL becomes basically impossible
    - I could still have the type checking at runtime
    - It would mean static types give no performance gain BUT they do give compile time type safety.

**REPL Static Typing**

- All REPL does is compile your new line and run it.
- All static typing does it keep some type-checking context
- So if I can separate that context, I can just pass it to the new compiler each time.
