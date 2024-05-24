use proc_macro::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttrStyle, Attribute, DeriveInput, FieldsNamed, Ident, Meta};

mod attrs;
use attrs::{ParseAttribute, FieldAttribute};

#[proc_macro_derive(MySqlBinder, attributes(sqlx_binder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let output = match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let mut named_mut = named.clone();
                let idents_filtered = named.iter()
                    .filter(|f| {
                        let attrs = attributes::<FieldAttribute>(&f.attrs);
                        match attrs.first() {
                            Some(attr) => !matches!(attr, FieldAttribute::Skip),
                            None => true,
                        }
                    });
                let idents_getfield = named_mut.iter_mut()
                    .filter_map(|f| {
                        let attrs = attributes::<FieldAttribute>(&f.attrs);
                        match attrs.first() {
                            Some(FieldAttribute::Skip) => None,
                            Some(FieldAttribute::Rename(val)) => Some(f.ident.as_ref().map(|id| Ident::new(val, id.span()))),
                            _ => Some(f.ident.clone()),
                        }
                    });
                
                let idents_enum = idents_filtered.clone().map(|f| &f.ident);
                let tys_enum = idents_filtered.clone().map(|f| &f.ty);

                let idents_getenum = idents_enum.clone();
                let idents_getenums = idents_enum.clone();
                let idents_bind = idents_enum.clone();

                let enumname = format_ident!("{}{}", ident, "FieldEnum");

                quote! {

                    impl #ident {

                        pub fn get_enum(&self, field_string: &str) -> Result<#enumname, String> {
                            match field_string {
                                #(stringify!(#idents_getenum) => {
                                    Ok(#enumname::#idents_getenum(self.#idents_getenum.clone()))
                                }),*
                                _ => Err(format!("invalid field name to getenum '{}'", field_string)),
                            }
                        }

                        /// return UpperCamelCase
                        pub fn get_struct_name(&self) -> &'static str {
                            stringify!(#ident)
                        }

                        /// return snake_case
                        pub fn get_struct_name_snake(&self) -> String {

                            let text = self.get_struct_name();

                            // The first character is never prepended with an underscore, so skip it even if it is an
                            // uppercase ASCII character.
                            let underscore_count = text.chars().skip(1).filter(|&c| c.is_ascii_uppercase()).count();
                            let mut result = String::with_capacity(text.len() + underscore_count);
                        
                            for (i, c) in text.chars().enumerate() {
                                if c.is_ascii_uppercase() {
                                    if i != 0 {
                                        result.push('_');
                                    }
                                    result.push(c.to_ascii_lowercase());
                                } else {
                                    result.push(c);
                                }
                            }
                        
                            result
                        }

                        pub fn get_field_names(&self) -> Vec<&'static str> {
                            vec![#(stringify!(#idents_getfield)),*]
                        }

                        pub fn get_field_enums(&self) -> Vec<#enumname> {
                            vec![#(#enumname::#idents_getenums(self.#idents_getenums.clone())),*]
                        }
                    }

                    #[derive(Debug, PartialEq, PartialOrd, Clone)]
                    #[allow(non_camel_case_types)]
                    pub enum #enumname{
                        #(#idents_enum(#tys_enum)),*
                    }

                    impl #enumname {
                        pub fn bind(
                            self, 
                            query: sqlx::query::Query<'_, sqlx::MySql, sqlx::mysql::MySqlArguments>, 
                        ) -> sqlx::query::Query<'_, sqlx::MySql, sqlx::mysql::MySqlArguments> {
                            match self {
                                #(#enumname::#idents_bind(p) => query.bind(p)),*
                            }
                        }
                    }
                }
            }
            syn::Fields::Unnamed(_) => panic!("Only NamedFields is supported"),
            syn::Fields::Unit => panic!("Only NamedFields is supported"),
        },
        syn::Data::Enum(_) => panic!("Enum is not supported. Only struct is supported"),
        syn::Data::Union(_) => panic!("Union is not supported. Only struct is supported"),
    };
    output.into()
}

fn attributes<A: ParseAttribute>(attrs: &[Attribute]) -> Vec<A> {
    let mut res = Vec::new();

    for attr in attrs {
        if attr.style != AttrStyle::Outer {
            continue;
        }

        let attr_name = attr
            .path
            .segments
            .iter()
            .last()
            .cloned()
            .expect("attribute is badly formatted");

        if attr_name.ident != "sqlx_binder" {
            continue;
        }

        let meta = attr
            .parse_meta()
            .expect("unable to parse attribute to meta");

        if let Meta::List(l) = meta {
            for arg in l.nested {
                res.push(A::parse(&arg));
            }
        }
    }

    res
}