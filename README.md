# magic-file-collection
克隆当前工程到本地
```
git clone https://github.com/chineseLiux/magic-file-collection.git
```
配置需要采集的文件路径,编辑 watching_path属性
```
vi config/settings.toml
```
测试，执行运行命令，控制行数据文件修改内容
```
cargo run
```

###### 配置文件 settings.toml 说明
| 分组     | 属性               | 名称          | 必填  | 说明            |
|--------|------------------|-------------|-----|---------------|
| log    | directory        | 目录          | 是   | ./log         |
| log    | file_name_prefix | 日志文件名称      | 是   | service.log   |
| notify | watching_path    | 监控文件路径      | 是   | ./data/offset |
| notify | flush_timing     | 文件读取偏移量刷盘时间 | 否   | 默认10秒         |
| notify | offset_file      | 偏移量记录文件路径   | 否   | ./data/offset |

