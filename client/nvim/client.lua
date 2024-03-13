local filetypes = { "html", "templ" }
local htmx_path = "/Users/hubertkowalski/Documents/Projects/htmx-ls/target/release/htmx-ls"

vim.lsp.set_log_level("debug")

local function setup()
	return {
		default_config = {
			cmd = { htmx_path },
			filetypes = filetypes,
			single_file_support = true,
			root_dir = vim.fs.dirname(vim.fs.find({ "index.html" }, { upward = true })[1]),
		},
		docs = {
			description = [[
https://github.com/hubcio2115/htmx-ls

Lsp is still very much work in progress and experimental. Use at your own risk.
]],
		},
	}
end

vim.lsp.start(setup().default_config)

vim.api.nvim_create_autocmd("LspAttach", {
	callback = function(_) end,
})
