-- Register the litigate grammar with nvim-treesitter so `:TSInstall litigate`
-- builds it. Works with both nvim-treesitter branches:
--   main:   require('nvim-treesitter.parsers') is a table of install specs,
--           re-read on the `User TSUpdate` event.
--   master: require('nvim-treesitter.parsers').get_parser_configs().
-- Neovim older than 0.12 loads only ABI 14 parsers, so the parser is then
-- regenerated from grammar.js (needs the tree-sitter CLI and node).
if vim.g.loaded_litigate then
  return
end
vim.g.loaded_litigate = true

local url = 'https://github.com/davidawad/litgraph'
local location = 'editors/tree-sitter-litigate'
local generate = vim.treesitter.language_version < 15

local function register()
  local ok, parsers = pcall(require, 'nvim-treesitter.parsers')
  if not ok then
    return
  end
  if type(parsers.get_parser_configs) == 'function' then
    parsers.get_parser_configs().litigate = {
      install_info = {
        url = url,
        branch = 'main',
        location = location,
        files = { 'src/parser.c' },
        requires_generate_from_grammar = generate,
      },
      filetype = 'litigate',
      maintainers = { '@davidawad' },
    }
  else
    parsers.litigate = {
      install_info = {
        url = url,
        branch = 'main',
        location = location,
        queries = location .. '/queries',
        generate = generate,
      },
      tier = 3,
    }
  end
end

register()
vim.api.nvim_create_autocmd('User', {
  pattern = 'TSUpdate',
  group = vim.api.nvim_create_augroup('litigate_ts_update', { clear = true }),
  callback = register,
})
