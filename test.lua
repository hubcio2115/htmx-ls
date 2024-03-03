local util = require("lspconfig.util")

local root_files = { "Cargo.toml" }
local htmx_path = "/Users/hubertkowalski/Documents/Projects/htmx-ls/target/release/htmx-ls"

local function setup()
  return {
    default_config = {
      name = "htmx-ls",
      cmd = { htmx_path },
      root_dir = vim.fs.dirname(vim.fs.find(root_files, { upward = true })[1]),
    },
  }
end

vim.lsp.start(setup().default_config)

vim.api.nvim_create_autocmd("LspAttach", {
  callback = function(args)
    print(vim.inspect(args))
  end,
})
