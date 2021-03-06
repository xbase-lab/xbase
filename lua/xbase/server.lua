local util = require "xbase.util"
local socket = require "xbase.socket"
local validate = vim.validate
local notify = require "xbase.notify"
local broadcast = require "xbase.broadcast"
local constants = require "xbase.constants"
local uv = vim.loop
local id = vim.loop.os_getpid()

---@class XBase
local M = {
  ---@type XBaseSocket @helper object to communcate with xbase daemon
  socket = nil,
  ---@type string[] @list of registered roots
  roots = {},
}

---Spawn xbase daemon in detached mode and executes cb on first stdout
---@param cb function
function M.spawn_daemon(cb)
  notify.info "Starting new dameon instance"
  local bin = constants.BIN_ROOT .. "/xbase"
  local stdout = uv.new_pipe()
  local _, _ = uv.spawn(bin, {
    stdio = { nil, stdout, nil },
    detached = true,
  })
  stdout:read_start(vim.schedule_wrap(function(_, _)
    M.socket = socket:connect(constants.SOCK_ADDR)
    stdout:read_stop()
    cb()
  end))
end

--- Ensure we have a connect socket and a running background daemon
function M.ensure_connection(cb)
  if M.socket == nil then
    if uv.fs_stat(constants.SOCK_ADDR) == nil then
      return M.spawn_daemon(cb)
    else
      M.socket = socket:connect(constants.SOCK_ADDR)
    end
  end
  cb()
end

---Send Request to socket, and on response call on_response with data if no error
---@param req table
---@param on_response? function(response:table)
function M.request(req, on_response)
  M.ensure_connection(function()
    M.socket:read_start(function(chunk)
      vim.schedule(function()
        local res = vim.json.decode(chunk)
        if res.error then
          notify.error(string.format("%s %s", res.error.kind, res.error.msg))
          return
        else
          if on_response then
            on_response(res.data)
          end
        end
      end)
      M.socket:read_stop()
    end)
    M.socket:write(req)
  end)
end

---Check whether the vim instance should be registered to xbase server.
---@param root string: current working directory
---@return boolean
function M.should_register(root)
  if uv.fs_stat(root .. "/project.yml") then
    return true
  elseif uv.fs_stat(root .. "/Project.swift") then
    return true
  elseif uv.fs_stat(root .. "/Package.swift") then
    return true
  elseif vim.fn.glob(root .. "/*.xcodeproj"):len() ~= 0 then
    return true
  end
  return false
end

---Register given root and return true if the root is registered
---@param root string
---@return boolean
function M.register(root)
  validate { root = { root, "string", false } }
  if M.roots[root] then
    return
  end

  require("xbase.logger").setup()

  local req = { method = "register", args = { id = id, root = root } }
  M.request(req, function(broadcast_address)
    broadcast.start(root, broadcast_address)
    M.roots[root] = true
  end)
end

---Drop a given root or drop all tracked roots if root is nil
---@param root string
function M.drop(root)
  M.request { method = "drop", args = { roots = { root } } }
end

return M
