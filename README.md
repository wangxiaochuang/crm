# rust template

## env

proto提醒错误，设置vscode配置

```json
"protoc": {
    "options": [
        "--proto_path=protos",
    ],
},
```

## postgres

### install

```
pip install pgcli
cargo install sqlx-cli --no-default-features --features rustls --features postgres
```

进入指定库

```sh
pgcli -h 127.0.0.1 -U postgres stats
```

创建、删除库

```sh
sqlx database drop -D postgres://postgres:postgres@127.0.0.1/stats
sqlx database create -D postgres://postgres:postgres@127.0.0.1/stats
```

创建迁移文件

```sh
sqlx migrate add initial

```

执行迁移文件

```sh
# 执行后，会在chat下创建一个名为_sqlx_migrations的表，记录了迁移的内容，多次执行不会有变化，如果文件改变会报错
# echo DATABASE_URL=postgres://postgres:postgres@127.0.0.1/chat > .env
# sqlx migrate run
sqlx migrate run -D postgres://postgres:postgres@127.0.0.1/stats
```

### 常见语法

创建数组字段

```sql
CREATE TABLE user_stats(
    ...
    recent_watched int[],
    ...
)
```

插入数组字段

```sql
insert into user_stats(recent_watched) values(array[1,2,3]::int[]);
```

添加某个元素是否在数组中作为条件

```sql
select name, email from user_stats where created_at > CURRENT_TIMESTAMP - in
 terval '100 days' and 164316 = any(recent_watched);
-- 或者 判断多个元素是否在某个字段内
select name, email from user_stats where created_at > CURRENT_TIMESTAMP - in
 terval '100 days' and array[164316] <@ recent_watched;
```

时间比较子句
```sql
select count(*) from user_stats where created_at > CURRENT_TIMESTAMP - inter
 val '100 days';
```

查看sql执行计划

```sql
explain SQL SENTENCE;
```
