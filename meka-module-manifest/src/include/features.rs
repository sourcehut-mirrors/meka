#[cfg(all(feature = "mlua-external", feature = "mlua-lua54"))]
let features = "mlua-lua54";
#[cfg(all(feature = "mlua-external", feature = "mlua-lua53"))]
let features = "mlua-lua53";
#[cfg(all(feature = "mlua-external", feature = "mlua-lua52"))]
let features = "mlua-lua52";
#[cfg(all(feature = "mlua-external", feature = "mlua-lua51"))]
let features = "mlua-lua51";
#[cfg(all(feature = "mlua-external", feature = "mlua-luajit"))]
let features = "mlua-luajit";
#[cfg(all(feature = "mlua-external", feature = "mlua-luajit52"))]
let features = "mlua-luajit52";
#[cfg(feature = "mlua-lua54")]
let features = "mlua-lua54,mlua-vendored";
#[cfg(feature = "mlua-lua53")]
let features = "mlua-lua53,mlua-vendored";
#[cfg(feature = "mlua-lua52")]
let features = "mlua-lua52,mlua-vendored";
#[cfg(feature = "mlua-lua51")]
let features = "mlua-lua51,mlua-vendored";
#[cfg(feature = "mlua-luajit")]
let features = "mlua-luajit,mlua-vendored";
#[cfg(feature = "mlua-luajit52")]
let features = "mlua-luajit52,mlua-vendored";
