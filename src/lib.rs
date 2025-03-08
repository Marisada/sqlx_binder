use proc_macro::{self, TokenStream};
use quote::{format_ident, quote};
use syn::{AttrStyle, Attribute, DeriveInput, FieldsNamed, Ident, Meta, parse_macro_input};

mod attrs;
use attrs::{FieldAttribute, FieldAttributes};

#[proc_macro_derive(MySqlBinder, attributes(sqlx_binder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let output = match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let mut named_mut = named.clone();
                let idents_filtered = named.iter().filter(|f| {
                    let attrs = attributes(&f.attrs);
                    match attrs.first() {
                        Some(attr) => !matches!(attr, FieldAttribute::Skip),
                        None => true,
                    }
                });
                let idents_getfield = named_mut.iter_mut().filter_map(|f| {
                    let attrs = attributes(&f.attrs);
                    match attrs.first() {
                        Some(FieldAttribute::Skip) => None,
                        Some(FieldAttribute::Rename(val)) => {
                            Some(f.ident.as_ref().map(|id| Ident::new(val, id.span())))
                        }
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

                        /// If `primary_key` is Some, will skip `primary_key` column.<br> 
                        /// If `custom_table_name` is None, will use struct name as table_name (automatically convert `PascalCase` to `snake_case`).<br>
                        /// - custom_table_name = `Some("some_table_name")`<br>
                        /// `extra_column` and `extra_statement` MUST have the same amount and start with `,` (or "" for empty).<br>
                        /// `extra_values` can be any type (MUST convert to `String` type) and have the same amount as `?` in `extra_column`.<br>
                        /// - extra_column = `,create_user,create_datetime,update_user,update_datetime,version`<br>
                        /// - extra_statement = `,?,now(),?,now(),1`<br>
                        /// - extra_values = `&["username", "username"]`
                        #[allow(clippy::too_many_arguments)]
                        pub async fn insert(
                            &self,
                            primary_key: Option<&str>,
                            custom_table_name: Option<&str>,
                            extra_column: &str,
                            extra_statement: &str,
                            extra_values: &[&str],
                            pool: &Pool<MySql>,
                            db_name: &str,
                        ) -> sqlx::Result<MySqlQueryResult> {

                            let tbname = custom_table_name.map(|s| s.to_string()).unwrap_or(self.get_struct_name_snake());
                            let mut keys = self.get_field_names();
                            let mut params = self.get_field_enums();

                            if let Some(pk) = primary_key {
                                let position = keys.iter().position(|k| *k == pk)
                                    .ok_or_else(|| sqlx::Error::ColumnNotFound(pk.to_string()))?;
                                let _removed_keys = keys.swap_remove(position);
                                let _removed_param = params.swap_remove(position);
                            }

                            let sql = [
                                "INSERT INTO ", db_name, ".", &tbname, " (",
                                    &keys.join(","), extra_column,
                                ") VALUE (",
                                    &vec!["?"; keys.len()].join(","), extra_statement,
                                ");"
                            ].join("");

                            let mut query = sqlx::query(&sql);
                            for param in params {
                                query = param.bind(query);
                            }
                            for extra_value in extra_values {
                                query = query.bind(extra_value);
                            }
                            query.execute(pool).await
                        }

                        /// `primary_key` using for `WHERE` in sql.<br> 
                        /// If `custom_table_name` is None, will use struct name as table_name (automatically convert `PascalCase` to `snake_case`)<br>
                        /// - custom_table_name = `Some("some_table_name")`<br>
                        /// `extra_column` MUST start with `,` (or "" for empty `extra_column`).<br>
                        /// `extra_values` can be any type (MUST convert to `String` type) and have the same amount as `?` in `extra_column`.<br>
                        /// - extra_column = `,update_user=?,update_datetime=now(),version=1`<br>
                        /// - extra_values = `&["username"]`
                        pub async fn update(
                            &self,
                            primary_key: &str,
                            custom_table_name: Option<&str>,
                            extra_column: &str,
                            extra_values: &[&str],
                            pool: &Pool<MySql>,
                            db_name: &str,
                        ) -> sqlx::Result<MySqlQueryResult> {

                            let tbname = custom_table_name.map(|s| s.to_string()).unwrap_or(self.get_struct_name_snake());
                            let mut keys = self.get_field_names();
                            let mut params = self.get_field_enums();

                            let position = keys.iter().position(|k| *k == primary_key)
                                .ok_or_else(|| sqlx::Error::ColumnNotFound(primary_key.to_string()))?;
                            let removed_keys = keys.swap_remove(position);
                            let removed_param = params.swap_remove(position);

                            let sql = [
                                "UPDATE ", db_name, ".", &tbname, " SET ",
                                &keys.iter().map(|k| [k, "=?"].join("")).collect::<Vec<String>>().join(","), extra_column,
                                " WHERE ", removed_keys, "=?;"
                            ].join("");

                            let mut query = sqlx::query(&sql);
                            for param in params {
                                query = param.bind(query);
                            }
                            for extra_value in extra_values {
                                query = query.bind(extra_value);
                            }
                            query = removed_param.bind(query);
                            query.execute(pool).await
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

fn attributes(attrs: &[Attribute]) -> Vec<FieldAttribute> {
    let mut res = Vec::new();

    for attr in attrs {
        if attr.style != AttrStyle::Outer {
            continue;
        }

        let attr_name = attr
            .path()
            .segments
            .iter()
            .last()
            .cloned()
            .expect("attribute is badly formatted");

        if attr_name.ident != "sqlx_binder" {
            continue;
        }

        if let Meta::List(list) = &attr.meta {
            match list.parse_args::<FieldAttributes>() {
                Ok(items) => res.extend(items.attrs),
                Err(e) => panic!("Error parsing field attributes: {}", e),
            }
        }
    }

    res
}
