use quote::{ToTokens, quote};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Read};
use std::path::{Path, PathBuf};
use std::string::String;

pub mod prelude {
    pub use crate::{Cat, CatKind, CatMap};
}

pub type CatMap<K> = HashMap<K, CatKind>;

#[derive(Clone)]
pub enum CatKind {
    // Normalizes all path types to PathBuf
    Path(PathBuf),
    // Normalizes all string types to String
    String(String),
    // Special case for compile-time strings
    Static(&'static str),
}

impl CatKind {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        CatKind::Path(path.as_ref().to_path_buf())
    }

    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        CatKind::String(s.as_ref().to_string())
    }

    pub fn from_static_str(s: &'static str) -> Self {
        CatKind::Static(s)
    }
}

impl ToTokens for CatKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expanded = match self {
            CatKind::Path(p) => {
                let path_str = p.to_string_lossy();
                quote! { ::meka::CatKind::Path(::std::path::PathBuf::from(#path_str)) }
            }
            CatKind::String(s) => {
                quote! { ::meka::CatKind::String(#s.to_string()) }
            }
            CatKind::Static(s) => {
                quote! { ::meka::CatKind::Static(#s) }
            }
        };
        tokens.extend(expanded);
    }
}

pub trait Cat {
    fn cat(&self) -> io::Result<String>;
}

impl Cat for CatKind {
    fn cat(&self) -> io::Result<String> {
        match self {
            CatKind::Path(p) => p.cat(),
            CatKind::String(s) => s.cat(),
            CatKind::Static(s) => s.cat(),
        }
    }
}

impl Cat for PathBuf {
    fn cat(&self) -> io::Result<String> {
        self.as_path().cat()
    }
}

impl Cat for &Path {
    fn cat(&self) -> io::Result<String> {
        let mut input = File::open(self)?;
        read_to_string(&mut input)
    }
}

impl Cat for Cow<'_, Path> {
    fn cat(&self) -> io::Result<String> {
        self.as_ref().cat()
    }
}

impl Cat for String {
    fn cat(&self) -> io::Result<String> {
        self.as_str().cat()
    }
}

impl Cat for &str {
    fn cat(&self) -> io::Result<String> {
        let mut input = Cursor::new(self);
        read_to_string(&mut input)
    }
}

impl Cat for Cow<'_, str> {
    fn cat(&self) -> io::Result<String> {
        self.as_ref().cat()
    }
}

fn read_to_string<R>(input: &mut R) -> io::Result<String>
where
    R: Read,
{
    let mut text = String::new();
    let mut reader = BufReader::new(input);
    reader.read_to_string(&mut text)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::env;
    use std::path::Path;

    use super::{Cat, CatKind, CatMap};

    #[test]
    fn it_works() {
        const ENV_VAR_OS_CARGO_MANIFEST_DIR: &str =
            "Unexpectedly could not read `CARGO_MANIFEST_DIR` environment variable";
        let mut cat_map: CatMap<Cow<'static, str>> = CatMap::new();
        cat_map.insert(Cow::from("Apr"), CatKind::Static("Showers"));
        cat_map.insert(
            Cow::from("May"),
            CatKind::Path(
                Path::new(&env::var_os("CARGO_MANIFEST_DIR").expect(ENV_VAR_OS_CARGO_MANIFEST_DIR))
                    .join("testdata")
                    .join("may.txt"),
            ),
        );
        assert_eq!(
            cat_map.get(&Cow::from("Apr")).unwrap().cat().unwrap(),
            String::from("Showers")
        );
        assert_eq!(
            cat_map
                .get(&Cow::from("May"))
                .unwrap()
                .cat()
                .unwrap()
                .trim_end(),
            String::from("Flowers")
        );
    }
}
