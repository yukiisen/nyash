[![progress-banner](https://backend.codecrafters.io/progress/shell/bcb2ac1e-5793-4748-a292-5fb9365a859f)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

This is my implementation for the challenge so far (I didn't peek at the solutions I promise ._.)

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
- Add inlint completions
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
