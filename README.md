<div align="center">
  <a href="https://github.com/ThePrimeagen/htmx-lsp#gh-light-mode-only"><img src="assets/logo.svg#gh-light-mode-only"        width="300px" alt="HTMX-LS logo"/></a>
  <a href="https://github.com/ThePrimeagen/htmx-lsp#gh-dark-mode-only"><img src="assets/logo.darkmode.svg#gh-dark-mode-only" width="300px" alt="HTMX-LS logo"/></a>
</div>

## HTMX-LS

This is so much a **work in progress**. At this point id does not allow any functionality of an lsp.

I've made it as my pet project for an OS competition. Hopefully someone will find it useful in the future.

## Integration

### Neovim

You have to build the project than source the file in `client/nvim/client.lua`

### V\*Code

The WIP extension can be found in `client/vscode`

## Development

### General

As of right now the general goal is just to provide completion for properties prefixed with `hx-` received without even looking at the context.

After that, would be to perform some code actions that make sense and allow for amazing utility around htmx.

### Build

```shell
cargo build

# OR auto-build on file save, requires `cargo-watch`
cargo install cargo-watch
cargo watch -x build
```
