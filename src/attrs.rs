use syn::{
    Expr, Lit, Meta, MetaNameValue,
    parse::{Parse, ParseStream},
};

#[derive(Debug)]
pub enum FieldAttribute {
    Skip,
    Rename(String),
}

pub struct FieldAttributes {
    pub attrs: Vec<FieldAttribute>,
}

impl Parse for FieldAttributes {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut attrs: Vec<FieldAttribute> = vec![];

        loop {
            if input.is_empty() {
                break;
            }

            let meta = input.parse::<Meta>()?;
            match meta {
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    if path.is_ident("rename") {
                        if let Expr::Lit(expr_lit) = value {
                            if let Lit::Str(val) = expr_lit.lit {
                                attrs.push(FieldAttribute::Rename(val.value()))
                            }
                        }
                    }
                }
                Meta::Path(path) if path.is_ident("skip") => attrs.push(FieldAttribute::Skip),
                u => panic!("unexpected '{:?}' attribute", u.path()),
            }
        }

        Ok(FieldAttributes { attrs })
    }
}
