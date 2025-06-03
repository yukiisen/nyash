[![progress-banner](https://backend.codecrafters.io/progress/shell/bcb2ac1e-5793-4748-a292-5fb9365a859f)]()

This is my implementation for the codecrafters' Create your own shell challenge so far (I didn't peek at the solutions I promise ._.),
it's still barebones, the source is messy and lasks proper testing but can easilly be improved.

Don't use this BTW, it's very unsafe..

I really enjoyed this by the way :)

# Features:
- Bash like tab completions
- Hard configuration
- History builtin (incomplete since they added the challenge recently)
- Double Or more Pipes (Double pipes probably need some check)
- FileDescriptor (stdin, stdout, stderr) redirect.
- No interpreter yet,
- No inline completions yet,

# Todo:
- Fix the cursor movement
- Add proper configuration
- Add inline completions
- Finish the history implementations
- Create a proper Lexical analyzer
- Crease a proper interpreter

# Usage:
Just clone, build, and run it:

```sh
git clone https://github.com/yukiisen/nyash
cd nyash
cargo run
```

If you want to install it anyway (I don't recommend)
```sh
cargo install --git https://github.com/yukiisen/nyash
```
