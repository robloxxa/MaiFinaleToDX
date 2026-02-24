use toml_edit::{
    visit_mut::{visit_table_like_kv_mut, VisitMut},
    Item, Value,
};

const MAX_INLINE_FIELDS: usize = 3;

pub struct FormatVisit;

impl VisitMut for FormatVisit {
    fn visit_table_like_kv_mut(&mut self, key: toml_edit::KeyMut<'_>, node: &mut Item) {
        into_table(node);

        if let Item::Table(table) = node {
            if is_small_leaf(table) {
                let mut inline = table.clone().into_inline_table();
                inline.fmt();
                *node = Item::Value(Value::InlineTable(inline));
                return;
            }
        }

        visit_table_like_kv_mut(self, key, node);
    }
}

fn is_small_leaf(table: &toml_edit::Table) -> bool {
    table.len() <= MAX_INLINE_FIELDS
        && table
            .iter()
            .all(|(_, v)| !v.is_table() && !v.is_array_of_tables() && !v.is_inline_table())
}

fn into_table(item: &mut Item) {
    if let Item::Value(Value::InlineTable(inline_table)) = item {
        let mut table = inline_table.clone().into_table();
        table.fmt();
        *item = toml_edit::Item::Table(table);
    }
}
