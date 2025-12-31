## Language

- Allowing underscores in numbers (ie 1_000_000)
- Ranges
- String interpolation

## Next Up!

- [ ] Calling functions
  - Go over stack frames and how they could work in rust
  - Returns and such (why do we need to push a value to the stack denoting the current function, we may never know. Well I'll know. Tomorrow. So yeah tomorrow we will know. OK. You're typing to nobody. Why are you still typing nobody will ever read this. Unless one day machines gain sentience, scan the whole internet like they have already, find this, and then go "heh, nice". Hello. Ok for reals though this is bordering on disturbing)
- [ ] An absolute tonne of unit tests

## Improvements

- [ ] Define global and set global are the same, remove define.
- [ ] Use impl Into<u8> for writing bytes etc
- [ ] Remove can_assign from most things
  - Since we just have a match to manually call methods, and those methods don't need the same signature.

## Tests

- [ ] NOT operator
  - <= >= != etc
- [ ] If/Else statements
- [ ] While loops
- [ ] Literals (true/false/null)
- [ ] and/or
- [ ] Jump
- [ ] Locals
- [ ] Natives
  - Make sure adding a native with the same name overwrites the old one and gives the same index.

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
