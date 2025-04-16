# 系统模式：MySQL-To-Excel 数据导出工具

## 系统架构

该工具是一个单体的命令行应用程序，遵循一个清晰、线性的处理流程：

1.  **初始化**:
    -   加载并解析 `config.toml` 配置文件。
    -   根据配置建立到 MySQL 数据库的异步连接池。
2.  **预处理**:
    -   获取一个数据库连接。
    -   执行 `COUNT(*)` 查询以确定总行数，用于分页计算和进度条初始化。
    -   执行一次查询以获取结果集的列名。
3.  **核心处理 (分页循环)**:
    -   根据总行数和配置的 `page_size` 计算总页数。
    -   循环遍历每一页：
        -   构造带有 `LIMIT` 和 `OFFSET` 的分页 SQL 查询。
        -   异步执行分页查询。
        -   异步迭代处理查询结果的每一行：
            -   将行中的每个 `Value` 转换为字符串。
            -   将转换后的字符串向量写入 Excel 工作表。
            -   更新进度条。
4.  **收尾**:
    -   完成进度条显示。
    -   保存 Excel 工作簿到可执行文件所在的目录 (`data.xlsx`)。
    -   释放数据库连接。

## 关键技术决策

-   **语言选择 (Rust)**: 选择 Rust 是为了获得内存安全、高性能和可靠的并发处理能力，特别适合处理可能很大的数据集。
-   **异步处理 (`tokio`, `mysql_async`)**: 使用 `async/await` 和 `tokio` 运行时以及 `mysql_async` 库来执行非阻塞的数据库操作，提高了 I/O 密集型任务（如网络查询）的效率。
-   **配置管理 (`toml`)**: 使用 TOML 格式的配置文件 (`config.toml`) 提供了一种清晰、易于人类阅读和编辑的方式来管理数据库凭据和查询参数。
-   **Excel 库 (`rust_xlsxwriter`)**: 选用 `rust_xlsxwriter` 来生成 `.xlsx` 文件，这是一个纯 Rust 实现，避免了外部依赖（如 libxlsxwriter C 库）。
-   **分页查询**: 采用 `LIMIT` 和 `OFFSET` 的数据库级分页是处理潜在大量数据的核心策略，避免一次性将所有数据加载到内存中。
-   **进度反馈 (`indicatif`)**: 集成 `indicatif` 库为长时间运行的任务提供了必要的视觉反馈。
-   **错误处理 (`anyhow`)**: 使用 `anyhow` 简化了错误处理和传播，提供了带上下文的错误信息。
-   **输出位置**: 将输出文件 `data.xlsx` 放置在可执行文件旁边，简化了用户的查找。

## 设计模式

-   **配置对象模式**: 将 `config.toml` 的内容反序列化为 Rust 结构体 (`Config`, `DatabaseConfig`, `QueryConfig`)，便于在代码中访问配置项。
-   **异步迭代器**: `mysql_async` 返回的查询结果 (`query_iter`) 是一个异步流，使用 `.for_each()` 进行异步处理。
-   **策略模式 (隐式)**: 分页大小 (`page_size`) 可以看作是一种策略参数，影响数据处理的粒度。
-   **单体脚本**: 整个逻辑包含在 `main.rs` 中，结构相对简单直接。

## 组件关系

```mermaid
graph TD
    A[main 函数] --> B(load_config);
    A --> C{mysql_async Pool};
    A --> D[COUNT(*) 查询];
    A --> E[获取列名查询];
    A --> F{rust_xlsxwriter Workbook/Worksheet};
    A --> G{indicatif ProgressBar};
    A --> H[分页查询循环];

    B -- 读取 --> I[config.toml];
    B -- 解析 --> J[Config Struct];
    A -- 使用 --> J;

    C -- 获取连接 --> K[mysql_async Connection];
    D -- 使用 --> K;
    E -- 使用 --> K;
    H -- 使用 --> K;

    E -- 列名 --> L(append_to_excel);
    H -- 数据行 --> M(convert_value_to_string);
    M -- 字符串 --> L;
    L -- 写入 --> F;

    D -- 总行数 --> G;
    H -- 更新 --> G;

    H -- 保存 --> F;
    A -- 获取路径 --> N(get_exe_dir);
    F -- 写入文件 --> O[data.xlsx @ exe_dir];
    N -- 提供路径 --> O;

    subgraph 外部库
        C; K; F; G; I; J;
    end

    subgraph 内部辅助函数
        B; L; M; N;
    end

    subgraph 核心逻辑
        A; D; E; H; O;
    end
```

-   `main` 是应用程序入口点和主要协调者。
-   `load_config` 负责加载和解析配置。
-   `mysql_async` (Pool, Connection, query_iter) 处理所有数据库交互。
-   `rust_xlsxwriter` (Workbook, Worksheet) 负责 Excel 文件的创建和写入。
-   `indicatif` (ProgressBar) 提供进度显示。
-   `convert_value_to_string` 和 `append_to_excel` 是数据处理和写入 Excel 的辅助函数。
-   `get_exe_dir` 用于确定输出文件的位置。
