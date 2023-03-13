# Bundle Lua

A crate to bundle lua files together, usage:

```bash
bundle-lua <OUTPUT> <SOURCE_DIR> <MAIN> <PACKAGES>...
```

`MAIN` and `PACKAGES`... are relative to `SOURCE_DIR`.

The arguments used in `PACKAGES` should be the arguments used in the `require`s.
