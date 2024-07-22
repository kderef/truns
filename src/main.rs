use std::str::FromStr;

mod value;
mod table;

fn main() {
    let input = r#"
    {
        "my_ass": true,
        "my_men": {"ass": true, "no": 1}
    }
    "#;

    let json_v = serde_json::Value::from_str(input).unwrap();
    let mut table = table::Table::from_json(json_v).unwrap();

    println!("{:#?}", table.items);

    println!("\nTo JSON: {:?}", table.to_yaml());
}
