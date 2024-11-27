# ha-ui

MVP UI, not tested on Windows, only on Linux.
It's build and runnable via [nix](https://nix.dev/install-nix.html), could probably
work in wsl. Just install nix and the following commands
should work:

Build:

```sh
nix build .#
```

Run:

```sh
nix run .#ha-ui

```

If you don't want to use nix, you need a rust compiler,
with the native package names on your distro you'll need to figure
them out yourself.
