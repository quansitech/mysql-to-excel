# MySQL-To-Excel 数据导出工具

一个用Rust编写的工具，用于从MySQL数据库分页查询数据并导出到Excel文件。

## 功能特性

- 支持分页查询大数据量，避免内存溢出
- 自动识别列名和数据格式
- 显示进度条，直观查看导出进度
- 生成标准Excel格式文件

## 运行环境

- Rust 1.65+ (仅编译时需要)
- MySQL 5.7+ 数据库

## 安装方式

### 1. 下载预编译版本

从Releases页面下载对应平台的预编译版本。

### 2. 从源码编译

```bash
git clone https://github.com/your-repo/mysql-to-excel.git
cd mysql-to-excel
cargo build --release
```

编译后的可执行文件位于 `target/release/mysql-to-excel`

## 配置说明

1. 复制 `config-sample.toml` 为 `config.toml`
2. 编辑 `config.toml` 文件：

```toml
[database]
host = "数据库地址"
port = 3306       # 数据库端口
user = "用户名"
password = "密码"
db_name = "数据库名"

[query]
sql = "SELECT * FROM your_table"  # 要执行的SQL查询
page_size = 100                  # 每页数据量(建议100-1000)
```

## 使用说明

1. 将 `config.toml` 与可执行文件放在同一目录
2. 运行程序：
   ```bash
   ./mysql-to-excel
   ```
3. 程序运行完成后，会在当前目录生成 `data.xlsx` 文件

## 输出文件格式

- 第一行为列名
- 后续每行为查询结果数据
- 自动转换MySQL数据类型为Excel格式

## 注意事项

1. 请确保SQL查询语句正确且权限足够
2. 大数据量导出时，建议设置合理的page_size(100-1000)
3. 程序运行期间请保持网络连接稳定
4. 导出的Excel文件会覆盖同名文件，请注意备份
