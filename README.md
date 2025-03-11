## Mysql-To-Excel

#### 使用说明

1. 将config-sample.toml改名为config.toml，配置数据库连接信息及需要导出的sql语句和page_size。page_size的大小可以根据数据量和数据库性能自行调整，建议不要设置太高，避免把数据库爬崩。

2. 去reeleases页面下载最新的mysql-to-excel执行文件。

3. config.toml文件必须和mysql-to-excel放到同一个目录下，然后执行mysql-to-excel，执行完后会在同目录下生成一个data.xlsx文件。