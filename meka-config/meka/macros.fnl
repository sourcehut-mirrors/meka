(local M {})

(fn M.manifest [...]
  "`(manifest ...)` instantiates new manifest with provided variable length arguments."
  `(let [val# ((. (require :meka) :manifest :new) ,...)]
     val#))

M
