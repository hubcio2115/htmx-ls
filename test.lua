local htmx_path = "/Users/hubertkowalski/Documents/Projects/htmx-ls/target/release/htmx-ls"

vim.lsp.start({
	name = "htmx-ls",
	cmd = { htmx_path },
	root_dir = vim.fs.dirname(vim.fs.find({ "Cargo.toml" }, { upward = true })[1]),
})

vim.api.nvim_create_autocmd("LspAttach", {
	callback = function(args)
		print(vim.inspect(args))
	end,
})
