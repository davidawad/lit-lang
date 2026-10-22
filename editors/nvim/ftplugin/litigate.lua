-- Buffer settings for litigate (.lit/.case). Two-space indentation is the
-- canonical format (`litigate fmt`, SPEC §10).
vim.bo.commentstring = '// %s'
vim.bo.comments = 's1:/*,mb:*,ex:*/,:///,://'
vim.bo.expandtab = true
vim.bo.shiftwidth = 2
vim.bo.softtabstop = 2
vim.bo.tabstop = 2
vim.opt_local.iskeyword:append('-')

-- Tree-sitter highlighting and folding when the parser is installed.
if pcall(vim.treesitter.start, 0, 'litigate') then
  vim.wo[0][0].foldmethod = 'expr'
  vim.wo[0][0].foldexpr = 'v:lua.vim.treesitter.foldexpr()'
  vim.wo[0][0].foldlevel = 99
  -- Indentation from queries/indents.scm needs nvim-treesitter's indentexpr.
  if pcall(require, 'nvim-treesitter') then
    vim.bo.indentexpr = "v:lua.require'nvim-treesitter'.indentexpr()"
  end
end

vim.b.undo_ftplugin = (vim.b.undo_ftplugin and (vim.b.undo_ftplugin .. ' | ') or '')
  .. 'setlocal commentstring< comments< expandtab< shiftwidth< softtabstop< tabstop< iskeyword< indentexpr<'
