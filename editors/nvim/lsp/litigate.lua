-- Language server config for Neovim 0.11+ (`:h lsp-config`). With this
-- directory on 'runtimepath', enable it with `vim.lsp.enable('litigate')`.
---@type vim.lsp.Config
return {
  cmd = { 'litigate', 'lsp', '--stdio' },
  filetypes = { 'litigate' },
  root_markers = { 'Lit.toml', 'Lit.lock', '.git' },
}
