use std::borrow::Cow;

pub type CatCowMap = io_cat::CatMap<Cow<'static, str>>;
