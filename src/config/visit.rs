use toml_edit::{
    visit_mut::{visit_table_like_kv_mut, VisitMut},
    Item, Value,
};

pub struct FormatVisit;

impl VisitMut for FormatVisit {
    fn visit_table_like_kv_mut(&mut self, key: toml_edit::KeyMut<'_>, node: &mut Item) {
        into_table(node);
        if matches!(key.get(), "p1_dx_touch_mapping" | "p2_dx_touch_mapping") {
            if let Item::Table(table) = node {
                table.iter_mut().for_each(|(_, v)| {
                    into_inline_table(v);
                })
            }

            return;
        }

        visit_table_like_kv_mut(self, key, node);
    }
}

fn into_inline_table(item: &mut Item) {
    if let Item::Table(table) = item {
        let mut table = table.clone().into_inline_table();
        table.fmt();
        *item = toml_edit::Item::Value(Value::InlineTable(table));
    }
}

fn into_table(item: &mut Item) {
    if let Item::Value(Value::InlineTable(inline_table)) = item {
        let mut table = inline_table.clone().into_table();
        table.fmt();
        *item = toml_edit::Item::Table(table);
    }
}
