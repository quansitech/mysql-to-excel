# 技术背景：MySQL-To-Excel 数据导出工具

## 核心技术栈

-   **编程语言**: Rust (Edition 2021)
    -   利用其内存安全、性能和并发特性。
-   **异步运行时**: Tokio (version 1.x, "full" features)
    -   为所有异步操作（主要是数据库 I/O）提供动力。
-   **构建系统**: Cargo
    -   用于依赖管理、编译、测试和构建。

## 主要依赖库 (Crates)

-   **`mysql_async` (0.30.0)**:
    -   用于与 MySQL 数据库进行异步交互。处理连接、查询执行和结果检索。
-   **`rust_xlsxwriter` (0.84.0)**:
    -   纯 Rust 实现的库，用于创建 `.xlsx` Excel 文件。负责工作簿、工作表和单元格的写入。
-   **`indicatif` (0.17)**:
    -   提供可定制的控制台进度条，用于显示数据导出进度。
-   **`serde` (1.0, "derive" feature)**:
    -   用于序列化和反序列化数据。在此项目中，主要用于将 `config.toml` 文件内容反序列化为 Rust 结构体 (`Config`)。
-   **`toml` (0.5)**:
    -   用于解析 TOML 格式的配置文件 (`config.toml`)。
-   **`anyhow` (1.0.97)**:
    -   提供灵活的错误处理机制，简化错误类型转换和上下文添加。
-   **`openssl` (0.10, "vendored" feature)**:
    -   提供 OpenSSL 绑定。`mysql_async` 可能依赖它来进行安全的 TLS/SSL 连接。使用 `vendored` 特性意味着 OpenSSL 库会与应用程序静态链接，简化了部署，但可能增加编译时间和二进制文件大小。

## 运行环境要求

-   **编译时**: Rust 1.65+ 工具链 (根据 `README.md`)。
-   **运行时**:
    -   访问目标 MySQL 5.7+ 数据库的网络连接。
    -   操作系统能够执行编译后的二进制文件。
    -   （可能）需要安装 OpenSSL 开发库，除非使用 `vendored` 特性成功静态链接。

## 开发与构建

-   **获取源码**: `git clone ...`
-   **构建**: `cargo build` (开发版本) 或 `cargo build --release` (优化版本)。
-   **运行**: 直接执行 `target/debug/mysql-to-excel` 或 `target/release/mysql-to-excel`。
-   **配置**: 需要在可执行文件同目录下放置一个有效的 `config.toml` 文件。

## 技术约束与考虑

-   **数据库兼容性**: 仅支持 MySQL。不支持其他 SQL 或 NoSQL 数据库。
-   **错误处理**: 依赖 `anyhow` 进行错误处理。主要的错误来源包括文件 I/O（读取配置）、网络问题（数据库连接）、SQL 错误（查询语法、权限）和 Excel 文件写入错误。
-   **性能**:
    -   分页查询是性能优化的关键，避免内存耗尽。
    -   异步 I/O 提高了效率。
    -   `page_size` 的选择会影响性能和内存使用之间的平衡。
-   **安全性**: 数据库密码明文存储在 `config.toml` 文件中，需要确保该文件的访问权限受到适当控制。数据库连接可能使用 SSL/TLS（由 `mysql_async` 和 `openssl` 处理），具体取决于 MySQL 服务器配置和连接选项。
-   **依赖管理**: 所有依赖项通过 `Cargo.toml` 管理。`Cargo.lock` 文件确保了构建的可重复性。
-   **OpenSSL 依赖**: `openssl` crate 是一个常见的复杂性来源，尤其是在跨平台编译或部署时。使用 `vendored` 特性旨在缓解这个问题。
