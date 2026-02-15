use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Expr, ExprArray,
};

/// Маппинг имён полей на их позиции
fn get_position(field_name: &str) -> Option<(usize, u8)> {
    match field_name {
        // A series
        "a1" => Some((1, 1)),
        "a2" => Some((1, 2)),
        "a3" => Some((1, 4)),
        "a4" => Some((1, 8)),
        "a5" => Some((1, 16)),
        "a6" => Some((2, 1)),
        "a7" => Some((2, 2)),
        "a8" => Some((2, 4)),

        // B series
        "b1" => Some((2, 8)),
        "b2" => Some((2, 16)),
        "b3" => Some((3, 1)),
        "b4" => Some((3, 2)),
        "b5" => Some((3, 4)),
        "b6" => Some((3, 8)),
        "b7" => Some((3, 16)),
        "b8" => Some((4, 1)),

        // D series
        "d1" => Some((4, 8)),
        "d2" => Some((4, 16)),
        "d3" => Some((5, 1)),
        "d4" => Some((5, 2)),
        "d5" => Some((5, 4)),
        "d6" => Some((5, 8)),
        "d7" => Some((5, 16)),
        "d8" => Some((6, 1)),

        // E series
        "e1" => Some((6, 2)),
        "e2" => Some((6, 4)),
        "e3" => Some((6, 8)),
        "e4" => Some((6, 16)),
        "e5" => Some((7, 1)),
        "e6" => Some((7, 2)),
        "e7" => Some((7, 4)),
        "e8" => Some((7, 8)),

        // C series
        "c1" => Some((4, 2)),
        "c2" => Some((4, 4)),

        _ => None,
    }
}

#[proc_macro_derive(AreaMapping, attributes(area))]
pub fn derive_area_mapping(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("AreaMapping can only be derived for structs with named fields"),
        },
        _ => panic!("AreaMapping can only be derived for structs"),
    };

    let mut field_data = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // Получаем позицию из маппинга
        let pos = get_position(&field_name_str)
            .unwrap_or_else(|| panic!("Unknown field name: {}", field_name_str));

        let mut activate_on = Vec::new();

        // Парсим атрибут #[area(activate_on = [...])]
        for attr in &field.attrs {
            if attr.path().is_ident("area") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("activate_on") {
                        let value = meta.value()?;
                        let expr: Expr = value.parse()?;

                        if let Expr::Array(ExprArray { elems, .. }) = expr {
                            for elem in elems {
                                activate_on.push(elem);
                            }
                        }
                        Ok(())
                    } else {
                        Ok(())
                    }
                })
                .unwrap();
            }
        }

        field_data.push((field_name.clone(), pos, activate_on));
    }

    // Генерируем Default impl
    let default_fields = field_data.iter().map(|(name, (pos_byte, pos_bit), activate_on)| {
        quote! {
            #name: Area::new(
                (#pos_byte, #pos_bit),
                vec![#(#activate_on),*],
                Duration::default(),
                Duration::default(),
            )
        }
    });

    // Генерируем From<BTreeMap> impl
    let from_map_arms = field_data.iter().map(|(name, (pos_byte, pos_bit), _)| {
        let name_upper = name.to_string().to_uppercase();

        quote! {
            #name_upper => mapping.#name = area.set_pos((#pos_byte, #pos_bit))
        }
    });

    // Генерируем into_values метод
    let field_count = field_data.len();
    let field_names = field_data.iter().map(|(name, _, _)| name);

    let expanded = quote! {
        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }

        impl From<BTreeMap<String, Area>> for #struct_name {
            fn from(map: BTreeMap<String, Area>) -> Self {
                let mut mapping = Self::default();

                map.into_iter().for_each(|(key, area)| match key.as_str() {
                    #(#from_map_arms,)*
                    _ => {}
                });

                mapping
            }
        }

        impl #struct_name {
            pub fn into_values(self) -> [Area; #field_count] {
                [#(self.#field_names),*]
            }
        }
    };

    TokenStream::from(expanded)
}
