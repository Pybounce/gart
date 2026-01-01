<div align="center">
  <img src="./logo.svg" height="340" width="340">
  <h1>gart</h1>
  <h3>A python-like scripted language named after the similarly underwhelming snake.</h3>
  <h5>Equipped with primitives (number, bool, string, null), first class functions and native functions, Gart lacks any of the depth found in a real language, but makes up for it with a cute logo.</h5>
</div>

```py
fn say_hello(name):
    print("hello " + name)

var name = input("Enter your name: ")

say_hello(name)
```
- **Gart is fast.** The virtual machine runs cache friendly, byte-sized operations with both locals and globals stored in densely packed vectors.
- **Gart is minimal.** It keeps Python‑like readability while adding only essential structure where it improves clarity or compilation, such as explicit `var` declarations.
- **Gart is safe.** Written in entirely safe Rust code, guaranteeing memory safety.
- **Gart is ergonomic.** The API is designed to be easy to embed and interact with, offering features like a step function that allows for better debugging, live visualization, and smooth frontend updates during execution.
<br></br>
## Usage

### Sandbox

The best way to use Gart is with the sandbox found at https://sandbox.skybounce.io.
- It comes with an array of examples and built in natives.
- More details can be found at https://github.com/Pybounce/sandbox.
<br></br>
### CLI

| Command                               | Description               |
| :------------------------------------ | :------------------------ |
| cargo run -r -- --path [file_path]    | Run any file.             |
| cargo run -r --example [example_name] | Run any built in example. |
| cargo run -r -- --help                | Display this message.     |

## Natives

- `time()`
  - Returns the time in seconds.
- `print(string)`
  - Prints the string to the output.
- `random_range(min, max)`
  - Requires arguments to be numbers, returns a random number between them (inclusive). Returns null otherwise.
- `number(val)`
  - Attempts to convert the value to a number and returns it. Returns null otherwise.
- `string(val)`
  - Attempts to convert the value to a string and returns it. Returns null otherwise.
- `input(msg)`
  - Prompts user with msg, returns the user input.
- `clear()`
  - Clears the output.
- `round(num)`
  - Returns the number rounded to the nearest integer. Returns null on failure.

> [!Note]
> Though all of these natives will be avaliable in the sandbox, some may be overriden to work with js, and new natives added.

