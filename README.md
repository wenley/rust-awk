# rust-awk
Rust Implementation of Awk

## What is this repo?

This started as a personal project to learn Rust. As such, I have no expectation to fully meet the official specification, but I will get as far as seems useful and informative for my learning. If folks find it interesting and want to help push it to proper completion, that would be a good motivator for turning this into something beyond a learning project.

### Contributing

Pull requests are welcome! There are a couple issues already filed that are good for getting started and familiar with the repo as it currently exists. I haven't yet set any guidelines for pull requests, and am happy to work with folks wherever they are. My one ask at this point is to run the following whenever making a change:

```
cargo build
cargo test
cargo fmt
# commit!
```

## Caveats

Since this started as a learning project, this is aimed to only support a subset of the full language specification. Notable omissions in its feature set:
- Functions (beyond printing)
  - Custom functions
  - Built-in functions
- Optional syntax elements
  - Semicolons to terminate statements are currently required
  - All items must have both a pattern and an action
  - Print statements must have parentheses
  - If statements must always come with an Else statement
- Convenience operators
  - Increment (`++`) and decrement operations (`--`)
  - Combo-Assignment operators (`+=`, `-=`)
  - C-style for loops
- "Advanced" features
  - Arrays
    - For-each loops
  - Manual `getline` ingestion
  - Assignment into Fields
  - From-Until pattern matching
  - Output redirection

All of these are candidates for being added if desired. To keep scope down for a V1, I'm considering them all out-of-scope until I finish the following:

- Rest of mathematical operators
  - Subtraction (`-`)
  - Multiplication (`*`)
  - Division (`/`)
  - Modulo-Division (`%`)
  - Exponentiation (`^`)
  - Comparisons (e.g. `<`, `==`, `>=`)
- While and Do-While loops
- Loop keywords
  - `break`
  - `continue`
  - `next`
- Boolean operators
  - And (`&&`)
  - Or (`||`)
  - Ternary (`? :`)
- Regex Match and Not-Match

See [Issues](https://github.com/wenley/rust-awk/issues) for the most up-to-date status on progress.

## Architecture

There isn't much surprising here, but for completeness

### Parsing

Parsing is done using [Nom](https://docs.rs/nom/5.1.2/nom/) and the standard patterns thereof. Since Nom spits out string slices, the copy of the full program string is stored in the program itself to make the lifetimes simpler without needing to make copies of all the String literals.

Expressions were the trickiest to parse due to their self-referential recursive nature. Parsing expressions was the trickiest bit for someone who hasn't written a parser before (read: me), since the naive solution easily leads to infinite recursion in parsing. [This blog post](https://craftinginterpreters.com/parsing-expressions.html) was the most helpful at solving this problem.

### Internal Representation and Evaluation

The program is held as a set of nested structs/enums. Again, the most important one to examine is `Expression`. In execution, expressions are also evaluated recursively in the natural way: doing a depth-first evaluation to evaluate leaf nodes, and combining those values together as determined by operators.

Awk has a few interesting properties in this regard:
1. There are only 3 types: Strings, Numbers, and "Uninitialized values"
2. Strings and Numbers can be coerced into the other. This coercion happens according to the way the value is being used. For example, addition is defined over numbers, so any value used in addition is first coerced to a number.

Aside from expression evaluation, the run time is straight forward: For every record, check every pattern and execute the associated action if the pattern evaluates to be true.

### Functional Core, Imperative Shell

The execution of the program is currently a mix of being imperative and functional. The only place doing printing is the `main()` function. However, all the execution performs mutations on the shared execution `struct Context` (holding variables). This may change in the future for "purity", but given the low-complexity of the overal program and that Rust already has nice protections around mutations, the benefits don't seem as big.
