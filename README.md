The tree-walk interpreter is now finished, and whatever tests I have run produce the correct results.
A few things that need to be done:

- Implement clock()
- Allow redeclarations in the global scope
- Garbage collection for environments
- Better error reporting
- Some actual test suite (integrate the test sets from the book's repository)
- Better code as I am still pretty new to Rust

I decided to start working on the bytecode VM before necessarily fixing all of the above.

