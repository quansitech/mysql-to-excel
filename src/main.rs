use mysql_async::{prelude::*, OptsBuilder, Pool, Value};
use serde::Deserialize;
use tokio;
use indicatif::ProgressBar;
use anyhow::Result;
<<<<<<< HEAD

use rust_xlsxwriter::*;
=======
use rust_xlsxwriter::*;
use clap::{Parser, Subcommand}; // Added clap imports

// Import the new module
mod excel_to_migration;
use excel_to_migration::ExcelToMigrationArgs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Export data from MySQL to an Excel file based on config.toml
    Export,
    /// Generate a Laravel migration file from an Excel file
    GenerateMigration(ExcelToMigrationArgs),
}

>>>>>>> master

#[derive(Deserialize, Debug)]
struct Config {
    database: DatabaseConfig,
    query: QueryConfig,
}

#[derive(Deserialize, Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
    user: String,
    password: String,
    db_name: String,
}

#[derive(Deserialize, Debug)]
struct QueryConfig {
    sql: String,
    page_size: usize,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
<<<<<<< HEAD
=======
    let cli = Cli::parse();

    match cli.command {
        Commands::Export => {
            run_export().await?;
        }
        Commands::GenerateMigration(args) => {
            excel_to_migration::run(args)?;
        }
    }

    Ok(())
}

// --- Existing export logic moved to its own function ---
async fn run_export() -> Result<(), anyhow::Error> {
    println!("Running MySQL to Excel export...");
>>>>>>> master
    let config = load_config()?;

    let opts = OptsBuilder::default()
        .ip_or_hostname(config.database.host)
        .tcp_port(config.database.port)
        .user(Some(config.database.user))
        .pass(Some(config.database.password))
        .db_name(Some(config.database.db_name));

    let pool = Pool::new(opts);
    let mut conn = pool.get_conn().await?;

    let total_rows: u64 = conn
        .query_first(&format!("SELECT COUNT(*) FROM ({}) AS subquery", config.query.sql))
        .await?
        .map(|r: mysql_async::Row| r.get(0).unwrap())
        .unwrap_or(0);

    let total_pages = (total_rows as f64 / config.query.page_size as f64).ceil() as u64;
    
    // 设置进度条样式
    let pb = ProgressBar::new(total_rows);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {percent}% {msg}")
            .unwrap()
            .progress_chars("##-")
    );
    pb.set_message("正在导出数据...");

    // 执行查询获取列名
    let query_result = conn.query_iter(&config.query.sql).await?;
    let columns = query_result.columns().unwrap();
    let column_names: Vec<String> = columns.iter().map(|c| c.name_str().to_string()).collect();

    let mut workbook = Workbook::new();
    let mut worksheet = workbook.add_worksheet();

    append_to_excel(&mut worksheet, column_names, 0)?;

    // 分页查询数据
    let mut index: u32 = 1;
    for page in 0..total_pages {
        let offset = page * config.query.page_size as u64;
        let paged_sql = format!(
            "{} LIMIT {} OFFSET {}",
            config.query.sql,
            config.query.page_size,
            offset
        );

        let mut rows = conn.query_iter(paged_sql).await?;
        rows.for_each(|row| {

            let values: Vec<String> = (0..columns.len())
                .map(|i| convert_value_to_string(row.get(i)))
                .collect();
            append_to_excel(&mut worksheet, values, index).unwrap();
            index += 1;
            pb.inc(1);
            
        }).await?;

        // 更新进度消息，显示当前页码
        pb.set_message(format!("处理中... 第 {}/{} 页", page + 1, total_pages));
    }
    
    pb.finish_with_message(format!("导出完成! 共处理 {} 条数据", total_rows));
    
    let excel_path = get_exe_dir()?.join("data.xlsx");
    workbook.save(excel_path)?;
    drop(conn);
    Ok(())
}

fn load_config() -> Result<Config, anyhow::Error> {
    let config_path = get_exe_dir()?.join("config.toml");
    let config_str = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

fn get_exe_dir() -> Result<std::path::PathBuf, anyhow::Error> {
    std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get executable directory"))
        .map(|p| p.to_path_buf())
}

fn append_to_excel(worksheet: &mut Worksheet, row: Vec<String>, row_index: u32) -> Result<(), anyhow::Error> {
    for (col_index, value) in row.iter().enumerate() {
        worksheet.write(row_index, col_index.try_into()?, value)?;
    }
    Ok(())
}

fn convert_value_to_string(value: Option<Value>) -> String {
    match value {
        Some(Value::Bytes(bytes)) => String::from_utf8_lossy(&bytes).to_string(),
        Some(Value::Int(x)) => x.to_string(),
        Some(Value::UInt(x)) => x.to_string(),
        Some(Value::Float(x)) => x.to_string(),
        Some(Value::Double(x)) => x.to_string(),
        Some(Value::Date(year, month, day, hour, minute, second, micro)) => {
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:06}.{:06}",
                year, month, day, hour, minute, second, micro)
        },
        Some(Value::Time(negative, days, hours, minutes, seconds, micros)) => {
            format!("{}{:03} {:02}:{:02}:{:06}.{:06}",
                if negative { "-" } else { "" }, days, hours, minutes, seconds, micros)
        },
        Some(Value::NULL) => "NULL".to_string(),
        None => "".to_string()
    }
}
