use syn::{Lit, NestedMeta, Meta, MetaNameValue};

pub trait ParseAttribute {
    fn parse(m: &NestedMeta) -> Self;
}

#[derive(Debug)]
pub enum FieldAttribute {
    Skip,
    Rename(String),
}

impl ParseAttribute for FieldAttribute {
    fn parse(m: &NestedMeta) -> Self {
        match m {
            NestedMeta::Meta(m) => match m {
                Meta::NameValue(MetaNameValue {
                    path,
                    lit: Lit::Str(val),
                    ..
                }) if path.is_ident("rename") => Self::Rename(val.value()),
                Meta::Path(path) if path.is_ident("skip") => Self::Skip,
                u => panic!("unexpected '{:?}' attribute", u.path()),
            },

            // NestedMeta::Meta(m) => match m.path().get_ident() {
            //     Some(i) if i == "skip" => Self::Skip,
            //     _ => panic!("unable to parse attribute"),
            // },

            NestedMeta::Lit(_) => panic!("unable to parse attribute"),
        }
    }
}
