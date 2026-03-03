use colored::Colorize;
use comfy_table::{Cell, Table};

pub fn run(query: &str, limit: usize, json: bool) -> anyhow::Result<()> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;
    let index_path = std::path::PathBuf::from(&home)
        .join(".skillguard")
        .join("index");

    let index = skillguard_registry::RegistryIndex::open(index_path);
    let results = index.search(query, limit)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }

    if results.is_empty() {
        println!("No skills found matching '{}'", query.yellow());
        return Ok(());
    }

    println!(
        "{} Search results for '{}':",
        "→".blue().bold(),
        query.cyan()
    );

    let mut table = Table::new();
    table.set_header(vec!["Name", "Version", "Description"]);

    for entry in &results {
        table.add_row(vec![
            Cell::new(&entry.name),
            Cell::new(&entry.version),
            Cell::new(entry.description.as_deref().unwrap_or("")),
        ]);
    }

    println!("{table}");

    Ok(())
}
