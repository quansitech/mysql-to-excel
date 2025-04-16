# MySQL-To-Excel 数据导出工具

一个用Rust编写的多功能工具，用于MySQL数据库和Excel文件之间的数据转换。

## 功能特性

- **MySQL到Excel导出**:
  - 支持分页查询大数据量，避免内存溢出
  - 自动识别列名和数据格式
  - 显示进度条，直观查看导出进度
  - 生成标准Excel格式文件

- **Excel到Laravel迁移**:
  - 从Excel文件生成Laravel迁移文件
  - 自动推断字段类型
  - 支持自定义表名
  - 可选添加自增主键和时间戳字段

## 运行环境

- Rust 1.65+ (仅编译时需要)
- MySQL 5.7+ 数据库 (仅MySQL到Excel功能需要)

## 安装方式

### 1. 下载预编译版本

从Releases页面下载对应平台的预编译版本。

### 2. 从源码编译

```bash
git clone https://github.com/your-repo/mysql-to-excel.git
cd mysql-to-excel
cargo build --release

##可兼容更多的linux版本
cargo build --release --target x86_64-unknown-linux-musl
```

编译后的可执行文件位于 `target/release/mysql-to-excel`

## 使用说明

本工具提供两个主要功能：MySQL到Excel导出和Excel到Laravel迁移。

### 1. MySQL到Excel导出

#### 配置说明

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

#### 执行导出

运行以下命令执行导出操作：

```bash
./mysql-to-excel export
```

程序运行完成后，会在当前目录生成 `data.xlsx` 文件。

#### 输出文件格式

- 第一行为列名
- 后续每行为查询结果数据
- 自动转换MySQL数据类型为Excel格式

### 2. Excel到Laravel迁移

此功能可以将Excel文件转换为Laravel迁移文件，方便数据库迁移。

#### 基本用法

```bash
./mysql-to-excel generate-migration -i 数据文件.xlsx
```

#### 高级选项

```bash
./mysql-to-excel generate-migration \
  -i 数据文件.xlsx \
  -o ./migrations \
  -t 自定义表名 \
  --with-pk \
  --with-timestamps \
  --chunk-size 200
```

参数说明：
- `-i, --input`: 输入Excel文件路径（必需）
- `-o, --output-dir`: 输出目录，默认为 `./migrations`
- `-t, --table`: 自定义表名，默认使用Excel文件名（转为snake_case）
- `--with-pk`: 添加自增主键 `id`
- `--with-timestamps`: 添加 `created_at` 和 `updated_at` 时间戳字段
- `--chunk-size`: 每个INSERT语句包含的行数，默认为100

#### 注意事项

- Excel文件的第一行必须是列标题
- 至少需要有一行数据用于推断字段类型
- 列名会自动转换为snake_case格式

## 性能优化建议

### MySQL到Excel导出

- **page_size设置**:
  - 小数据量（<10万行）：可设置较大值（500-1000）
  - 大数据量（>10万行）：建议设置较小值（100-300）
  - 内存受限环境：设置更小的值（50-100）

- **SQL查询优化**:
  - 尽量使用索引字段进行查询
  - 避免使用 `SELECT *`，只选择需要的列
  - 考虑在SQL中进行预处理（如格式化、计算等）

### Excel到Laravel迁移

- 对于大型Excel文件，建议增加chunk_size值（200-500）
- 如果只需要结构而不需要数据，可以使用较小的Excel样本文件

## 常见问题解答

### 1. 数据库连接失败

**问题**: 程序报错无法连接到数据库。

**解决方案**:
- 检查config.toml中的数据库连接信息是否正确
- 确认MySQL服务器是否运行
- 检查网络连接和防火墙设置
- 验证用户名和密码是否正确

### 2. 内存使用过高

**问题**: 导出大数据量时程序内存使用过高。

**解决方案**:
- 减小config.toml中的page_size值
- 优化SQL查询，只选择必要的列
- 考虑分多次导出（使用WHERE子句限制范围）

### 3. 导出速度慢

**问题**: 数据导出速度较慢。

**解决方案**:
- 增大page_size值（在内存允许的情况下）
- 优化SQL查询，添加适当的索引
- 检查网络连接质量
- 确保数据库服务器负载不过高

### 4. 数据类型转换问题

**问题**: 某些特殊数据类型在Excel中显示不正确。

**解决方案**:
- 在SQL查询中使用CAST或CONVERT函数预处理数据
- 对于日期时间类型，考虑使用特定的格式化函数

## 注意事项

1. 请确保SQL查询语句正确且权限足够
2. 大数据量导出时，建议设置合理的page_size(100-1000)
3. 程序运行期间请保持网络连接稳定
4. 导出的Excel文件会覆盖同名文件，请注意备份
5. 配置文件中的数据库密码是明文存储，请注意安全
