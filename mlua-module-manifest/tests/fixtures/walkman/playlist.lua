local M = {}

M.new = function(...)
  return setmetatable({...}, {__name = "playlist"})
end

return M
