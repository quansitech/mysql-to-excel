use anyhow::{Context, Result};
// Simplify imports, rely on DataType::Variant
use calamine::{open_workbook, Data, Range, Reader, Xlsx};
use chrono::Local;
use heck::ToUpperCamelCase;
use clap::Parser;
use heck::ToSnakeCase;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct ExcelToMigrationArgs {
    /// Path to the input Excel file.
    #[clap(short, long, value_parser)]
    pub input: PathBuf,

    /// Directory to output the migration file. Defaults to './migrations'.
    #[clap(short = 'o', long, value_parser, default_value = "./migrations")]
    pub output_dir: PathBuf,

    /// Optional table name. Defaults to the input filename (snake_case).
    #[clap(short, long, value_parser)]
    pub table: Option<String>,

    /// Add auto-incrementing primary key 'id'.
    #[clap(long, action)]
    pub with_pk: bool,

    /// Add 'created_at' and 'updated_at' timestamps.
    #[clap(long, action)]
    pub with_timestamps: bool,

    /// Number of rows per insert chunk.
    #[clap(long, value_parser, default_value_t = 100)]
    pub chunk_size: usize,
}

pub fn run(args: ExcelToMigrationArgs) -> Result<()> {
    println!("Starting Excel to Laravel Migration conversion...");
    println!("Input file: {:?}", args.input);
    println!("Output directory: {:?}", args.output_dir);

    let table_name = get_table_name(&args)?;
    let (column_names, column_types, range) = process_excel_data(&args.input)?;
    let migration_path = generate_migration_file(&args, &table_name, &column_names, &column_types, &range)?;

    println!("Migration file generated successfully: {:?}", migration_path);
    Ok(())
}

fn get_table_name(args: &ExcelToMigrationArgs) -> Result<String> {
    let table_name = args.table.clone().unwrap_or_else(|| {
        args.input
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("default_table")
            .to_snake_case()
    });
    println!("Target table name: {}", table_name);
    Ok(table_name)
}

fn process_excel_data(
    input_path: &Path
) -> Result<(Vec<String>, Vec<String>, Range<Data>)> {
    let mut workbook: Xlsx<_> = open_workbook(input_path)
        .context(format!("Failed to open Excel file: {:?}", input_path))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .context("Excel file has no sheets")?
        .clone();
    
    let range = workbook
        .worksheet_range(&sheet_name)
        .with_context(|| format!("Could not access sheet '{}'", sheet_name))?;

    if range.height() < 2 {
        anyhow::bail!(
            "Excel sheet '{}' must have at least 2 rows (header + data).", 
            sheet_name
        );
    }

    let column_names = extract_column_names(&range)?;
    let column_types = infer_column_types(&range, &column_names)?;

    Ok((column_names, column_types, range))
}

fn extract_column_names(range: &Range<Data>) -> Result<Vec<String>> {
    let header_row = range.rows().next().context("Failed to get header row")?;
    
    let headers_raw: Vec<String> = header_row
        .iter()
        .map(cell_to_string)
        .collect();

    let column_names: Vec<String> = headers_raw
        .iter()
        .map(|h| h.trim().to_snake_case())
        .collect();

    println!("Normalized column names: {:?}", column_names);
    Ok(column_names)
}

fn infer_column_types(range: &Range<Data>, column_names: &[String]) -> Result<Vec<String>> {
    let first_data_row = range.rows().nth(1).context("Failed to get first data row")?;

    if column_names.len() != first_data_row.len() {
        anyhow::bail!(
            "Header count ({}) does not match data row count ({}).",
            column_names.len(),
            first_data_row.len()
        );
    }

    let column_types: Vec<String> = first_data_row
        .iter()
        .map(infer_laravel_type)
        .collect();

    println!("Inferred column types: {:?}", column_types);
    Ok(column_types)
}

fn generate_migration_file(
    args: &ExcelToMigrationArgs,
    table_name: &str,
    column_names: &[String],
    column_types: &[String],
    range: &Range<Data>,
) -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y_%m_%d_%H%M%S");
    let migration_name = format!("{}_{}_table", timestamp, table_name);
    let migration_file_name = format!("{}.php", migration_name);
    let migration_path = args.output_dir.join(migration_file_name);
    let class_name = format!("{}Table", table_name.to_upper_camel_case());

    fs::create_dir_all(&args.output_dir)
        .context(format!("Failed to create output directory: {:?}", args.output_dir))?;

    let mut file = File::create(&migration_path)
        .context(format!("Failed to create migration file: {:?}", migration_path))?;

    let template = fs::read_to_string("template.php")
        .context("Failed to read template.php file")?;

    let schema_columns = generate_schema_columns(column_names, column_types, range)?;
    let data_insert = generate_data_inserts(table_name, column_names, range, args.chunk_size)?;

    let content = template
        .replace("TemplateMigration", &class_name)
        .replace(
            "// up()", 
            &format!(
                "Schema::create('{}', function (Blueprint $table) {{\n{}{}{}}});\n\n{}",
                table_name,
                schema_columns,
                if args.with_pk { "            $table->id();\n" } else { "" },
                if args.with_timestamps { "            $table->timestamps();\n" } else { "" },
                data_insert
            )
        )
        .replace(
            "// down()", 
            &format!("Schema::dropIfExists('{}');", table_name)
        );

    file.write_all(content.as_bytes())
        .context(format!("Failed to write migration file: {:?}", migration_path))?;

    Ok(migration_path)
}

fn generate_schema_columns(
    column_names: &[String],
    column_types: &[String],
    range: &Range<Data>,
) -> Result<String> {
    let first_data_row = range.rows().nth(1).context("No data rows found")?;
    
    let schema = column_names.iter().zip(column_types.iter()).enumerate()
        .map(|(i, (col_name, col_type))| {
            let mut line = format!("            $table->{}('{}')", col_type, col_name);
            if let Some(cell) = first_data_row.get(i) {
                if matches!(cell, Data::Empty) {
                    line.push_str("->nullable()");
                }
            }
            line.push_str(";\n");
            line
        })
        .collect::<String>();

    Ok(schema)
}

fn generate_data_inserts(
    table_name: &str,
    column_names: &[String],
    range: &Range<Data>,
    chunk_size: usize,
) -> Result<String> {
    if range.height() <= 1 {
        return Ok(String::new());
    }

    let mut inserts = String::new();
    inserts.push_str("        // Insert data\n");
    inserts.push_str("        $data = [\n");

    for (row_idx, row) in range.rows().skip(1).enumerate() {
        if row_idx % chunk_size == 0 && row_idx > 0 {
            inserts.push_str("        ];\n\n        DB::table('");
            inserts.push_str(table_name);
            inserts.push_str("')->insert($data);\n\n        $data = [\n");
        }

        inserts.push_str("            [\n");
        for (i, cell) in row.iter().enumerate() {
            let col_name = &column_names[i];
            let value = match cell {
                Data::String(s) => format!("'{}'", s.replace('\'', "\\'")),
                Data::Int(i) => i.to_string(),
                Data::Float(f) => f.to_string(),
                Data::Bool(b) => if *b { "true" } else { "false" }.to_string(),
                Data::DateTime(_) | Data::DateTimeIso(_) => format!("'{}'", cell_to_string(cell)),
                Data::Empty => "null".to_string(),
                _ => format!("'{}'", cell_to_string(cell)),
            };
            inserts.push_str(&format!("                '{}' => {},\n", col_name, value));
        }
        inserts.push_str("            ],\n");
    }

    inserts.push_str(&format!("        ];\n\n        DB::table('{}')->insert($data);\n", table_name));
    Ok(inserts)
}

// --- Helper Functions ---

/// Converts a Calamine DataType to a String representation.
// Restore type annotation, use DataType::Variant
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::String(s) => s.trim().to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(dt) => dt.to_string(),
        Data::DurationIso(d) => d.to_string(),
        Data::Error(e) => format!("Error: {:?}", e),
        Data::Empty => String::new(),
    }
}

/// Infers a simplified Laravel migration type based on the cell's DataType.
// Restore type annotation, use DataType::Variant
fn infer_laravel_type(cell: &Data) -> String {
    match cell {
        Data::Int(_) => "integer".to_string(),
        Data::Float(_) => "decimal".to_string(),
        Data::DateTime(_) | Data::DateTimeIso(_) => "dateTime".to_string(),
        Data::Bool(_) => "boolean".to_string(),
        // Default to string for others (String, Empty, Error, Duration*)
        _ => "string".to_string(),
    }
}
