-- Headless test for editors/nvim; run by `cargo nextest run -p litigate-editors`
-- (editors/litigate-editors/tests/nvim.rs).
local repo = assert(os.getenv('LITIGATE_REPO'), 'LITIGATE_REPO unset')
local parser_path = assert(os.getenv('LITIGATE_PARSER'), 'LITIGATE_PARSER unset')

local passed, failed = 0, 0
local function test(name, fn)
  local ok, err = pcall(fn)
  if ok then
    passed = passed + 1
    io.stdout:write('ok   ' .. name .. '\n')
  else
    failed = failed + 1
    io.stdout:write('FAIL ' .. name .. ': ' .. tostring(err) .. '\n')
  end
end

local function corpus()
  local out = vim.fn.systemlist({
    'find', repo,
    '(', '-name', '.git', '-o', '-name', 'target', '-o', '-name', 'node_modules', '-o', '-name', '.worktrees', ')',
    '-prune', '-o', '-type', 'f', '(', '-name', '*.lit', '-o', '-name', '*.case', ')', '-print',
  })
  table.sort(out)
  return out
end

local function captures_at(buf, row, col)
  local names = {}
  for _, c in ipairs(vim.treesitter.get_captures_at_pos(buf, row, col)) do
    names[#names + 1] = c.capture
  end
  return names
end

assert(vim.treesitter.language.add('litigate', { path = parser_path }))
local files = corpus()

test('corpus is non-empty (' .. #files .. ' files)', function()
  assert(#files >= 2, 'found ' .. #files)
end)

test('every corpus file: filetype, ftplugin, clean parse, highlighter', function()
  for _, file in ipairs(files) do
    vim.cmd.edit(vim.fn.fnameescape(file))
    local buf = vim.api.nvim_get_current_buf()
    assert(vim.bo[buf].filetype == 'litigate', file .. ': filetype ' .. vim.bo[buf].filetype)
    assert(vim.bo[buf].commentstring == '// %s', file .. ': ftplugin not applied')
    assert(vim.bo[buf].shiftwidth == 2, file .. ': shiftwidth')
    assert(vim.treesitter.highlighter.active[buf], file .. ': highlighter not started')
    local tree = vim.treesitter.get_parser(buf, 'litigate'):parse()[1]
    assert(not tree:root():has_error(), file .. ': parse error')
    vim.cmd('bwipeout!')
  end
end)

test('every query compiles and runs over the corpus', function()
  for _, name in ipairs({ 'highlights', 'indents', 'locals', 'folds', 'injections' }) do
    local query = vim.treesitter.query.get('litigate', name)
    assert(query, name .. ': not found on runtimepath')
    for _, file in ipairs(files) do
      local source = table.concat(vim.fn.readfile(file), '\n')
      local root = vim.treesitter.get_string_parser(source, 'litigate'):parse()[1]:root()
      for _ in query:iter_captures(root, source) do
      end
    end
  end
end)

-- The tree-sitter highlight assertions (`// <- capture`, `//   ^ capture`)
-- must hold in Neovim too: the expected capture is the last one at the
-- position, since later patterns win.
test('highlight assertions in tree-sitter-litigate/test/highlight', function()
  local dir = repo .. '/editors/tree-sitter-litigate/test/highlight/'
  local checked = 0
  for _, name in ipairs({ 'sample.lit', 'sample.case' }) do
    vim.cmd.edit(vim.fn.fnameescape(dir .. name))
    local buf = vim.api.nvim_get_current_buf()
    vim.treesitter.get_parser(buf, 'litigate'):parse()
    local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
    local target
    for i, line in ipairs(lines) do
      local slash = line:find('//', 1, true)
      local arrow, expect = line:match('^%s*//%s*(<%-)%s*([%w%._]+)%s*$')
      local caret_col, caret_expect = line:match('^%s*//()%s*%^%s*([%w%._]+)%s*$')
      if arrow then
        local names = captures_at(buf, target, slash - 1)
        assert(names[#names] == expect, ('%s:%d: <- %s, got %s'):format(name, i, expect, vim.inspect(names)))
        checked = checked + 1
      elseif caret_col then
        local col = line:find('^', 1, true) - 1
        local names = captures_at(buf, target, col)
        assert(names[#names] == caret_expect, ('%s:%d: ^ %s, got %s'):format(name, i, caret_expect, vim.inspect(names)))
        checked = checked + 1
      else
        target = i - 1
      end
    end
    vim.cmd('bwipeout!')
  end
  assert(checked >= 20, 'only ' .. checked .. ' assertions found')
  io.stdout:write('     (' .. checked .. ' highlight assertions)\n')
end)

test('folds come from queries/folds.scm', function()
  vim.cmd.edit(vim.fn.fnameescape(repo .. '/lit/examples/rule12.lit'))
  local buf = vim.api.nvim_get_current_buf()
  assert(vim.wo.foldmethod == 'expr')
  vim.treesitter.get_parser(buf, 'litigate'):parse()
  local row = vim.fn.search('exclude_trigger_day')
  assert(row > 0)
  assert(vim.fn.foldlevel(row) >= 1, 'foldlevel ' .. vim.fn.foldlevel(row))
  assert(vim.fn.foldlevel(1) == 0)
  vim.cmd('bwipeout!')
end)

test('lsp/litigate.lua config', function()
  local config = dofile(repo .. '/editors/nvim/lsp/litigate.lua')
  assert(vim.deep_equal(config.cmd, { 'litigate', 'lsp', '--stdio' }))
  assert(vim.deep_equal(config.filetypes, { 'litigate' }))
  if vim.fn.has('nvim-0.11') == 1 then
    assert(vim.deep_equal(vim.lsp.config.litigate.cmd, config.cmd), 'lsp/ not discovered on runtimepath')
  end
end)

test('nvim-treesitter registration (master and main APIs)', function()
  local plugin = repo .. '/editors/nvim/plugin/litigate.lua'
  local configs = {}
  package.loaded['nvim-treesitter.parsers'] = { get_parser_configs = function() return configs end }
  vim.g.loaded_litigate = nil
  dofile(plugin)
  local master = configs.litigate
  assert(master and master.filetype == 'litigate')
  assert(master.install_info.url == 'https://github.com/davidawad/litgraph')
  assert(master.install_info.location == 'editors/tree-sitter-litigate')

  local main = {}
  package.loaded['nvim-treesitter.parsers'] = main
  vim.api.nvim_exec_autocmds('User', { pattern = 'TSUpdate' })
  assert(main.litigate and main.litigate.install_info.queries == 'editors/tree-sitter-litigate/queries')
  for _, path in ipairs({ 'src/parser.c', 'src/scanner.c', 'queries/highlights.scm' }) do
    assert(vim.uv.fs_stat(repo .. '/' .. main.litigate.install_info.location .. '/' .. path), path)
  end
  package.loaded['nvim-treesitter.parsers'] = nil
end)

io.stdout:write(('%d passed, %d failed\n'):format(passed, failed))
vim.cmd(failed == 0 and 'qa!' or 'cquit 1')
