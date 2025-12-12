## Language

- Allowing underscores in numbers (ie 1_000_000)
- Ranges
- String interpolation

## Next Up!

- [ ] Literal compiler support (null/true/false)
- [ ] String support
  - Which means working out objects...end me pls.
- [ ] Global variables

**Global Variables**

- The basic idea is that, in the vm, globals are stored in a single vec
- GetGlobal and SetGlobal will have an opcode for the index into this vec, similar to locals
- For this the compile will need to do extra work.
- _2 Pass Compiler_
  - Run through tokens and only do global declarations
  - Means keeping track of the current depth etc.
  - When you declare a global
    - add it to the vec
    - and also have a hashmap (global_name => index)
  - Second pass does everything except global definitions
  - When it finds something trying to access a global, it uses the index in the hashmap.
- _Patching_
  - This lets the compiler run through everything
  - When it encounters trying to get an undefined identifier, it stores the info in a hashmap shown below
    - (IdentifierName => Vec<(byteOffset, token)>)
  - When it finds a global declaration, it stores it in hash (name => index)
  - At the end of compilation, it runs through the patch hashmap, and corrects all that it can.
    - If it encounters one it cannot patch, it raises an error using the token

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
