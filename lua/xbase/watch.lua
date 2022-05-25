local lib = require "libxbase"
local config = require("xbase.config").values
local M = {}

M.is_watching = function(config, command)
  local root = vim.loop.cwd()
  local watching = vim.g.xbase.watcher[root]
  local key = string.format("%s:%s:xcodebuild -configuration %s", root, command, config.configuration)

  if config.sysroot then
    key = key .. " -sysroot " .. config.sysroot
  end

  if config.scheme then
    key = key .. " -scheme " .. config.scheme
  end

  key = key .. " -target " .. config.target

  return watching[key] ~= nil
end

M.start = function(opts)
  lib.watch_target { config = opts, ops = "Start" }
end

M.stop = function(opts)
  lib.watch_target { config = opts, ops = "Stop" }
end

M.feline_provider = function()
  return {
    provider = function(_)
      icon = {}
      --- TODO(nvim): only show build status in xcode supported files?
      local config = config.statusline
      local status = vim.g.xbase_watch_build_status

      if status == "running" then
        icon.str = config.running.icon
        icon.hl = { fg = config.running.color }
      elseif status == "success" then
        icon.str = config.success.icon
        icon.hl = { fg = config.success.color }
      elseif status == "failure" then
        icon.str = config.failure.icon
        icon.hl = { fg = config.failure.color }
      else
        icon.str = " "
      end

      if icon.str == " " then
        return " ", icon
      else
        icon.str = " [" .. icon.str .. " xcode]"
        return " ", icon
      end
    end,

    hl = {},
  }
end

return M
