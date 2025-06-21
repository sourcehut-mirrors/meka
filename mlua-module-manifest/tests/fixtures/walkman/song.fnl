(local M {})

(fn M.new [artist album title]
  (setmetatable {: artist : album : title} {:__name :song}))

M
