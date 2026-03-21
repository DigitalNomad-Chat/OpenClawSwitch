# 模板 002: SQL查询模板

**适用场景：** 数据提取、数据分析、报表生成
**数据库类型：** MySQL / PostgreSQL

---

## 基础查询模板

### 模板1：数据查询

```sql
-- 基础查询
SELECT
    column1,
    column2,
    column3
FROM table_name
WHERE condition
ORDER BY column_name DESC
LIMIT 100;
```

### 模板2：条件查询

```sql
-- 多条件查询
SELECT *
FROM table_name
WHERE condition1
  AND condition2
  OR condition3
ORDER BY column_name;
```

### 模板3：范围查询

```sql
-- 日期范围查询
SELECT *
FROM table_name
WHERE date_column >= '{start_date}'
  AND date_column < '{end_date}';

-- 数值范围查询
SELECT *
FROM table_name
WHERE value_column BETWEEN {min_value} AND {max_value};
```

---

## 聚合查询模板

### 模板4：分组聚合

```sql
-- 单维度聚合
SELECT
    category_column,
    COUNT(*) as count,
    SUM(amount) as total,
    AVG(amount) as average
FROM table_name
GROUP BY category_column
ORDER BY total DESC;
```

### 模板5：多维度聚合

```sql
-- 多维度聚合
SELECT
    category1,
    category2,
    COUNT(*) as count,
    SUM(amount) as total
FROM table_name
WHERE condition
GROUP BY category1, category2
ORDER BY total DESC;
```

### 模板6：HAVING过滤

```sql
-- 聚合后过滤
SELECT
    category,
    COUNT(*) as count
FROM table_name
GROUP BY category
HAVING COUNT(*) > {threshold};
```

---

## 连接查询模板

### 模板7：内连接

```sql
-- 内连接
SELECT
    a.column1,
    a.column2,
    b.column1
FROM table_a a
JOIN table_b b ON a.id = b.id
WHERE condition
ORDER BY a.column1;
```

### 模板8：左连接

```sql
-- 左连接
SELECT
    a.*,
    b.column1
FROM table_a a
LEFT JOIN table_b b ON a.id = b.id
WHERE condition;
```

### 模板9：自连接

```sql
-- 自连接
SELECT
    a.user_id as user1,
    b.user_id as user2
FROM users a
JOIN users b ON a.referrer_id = b.user_id;
```

---

## 高级查询模板

### 模板10：子查询

```sql
-- 子查询
SELECT
    user_id,
    user_name,
    (SELECT COUNT(*) FROM orders WHERE user_id = u.user_id) as order_count
FROM users u
WHERE user_id IN (SELECT DISTINCT user_id FROM orders WHERE amount > 100);
```

### 模板11：窗口函数

```sql
-- 排名
SELECT
    user_id,
    amount,
    ROW_NUMBER() OVER (ORDER BY amount DESC) as rank,
    RANK() OVER (ORDER BY amount DESC) as rank_dense,
    DENSE_RANK() OVER (ORDER BY amount DESC) as rank_dense_no_gap
FROM table_name;
```

### 模板12：CASE表达式

```sql
-- CASE表达式
SELECT
    column1,
    CASE
        WHEN condition1 THEN result1
        WHEN condition2 THEN result2
        ELSE default_result
    END as new_column
FROM table_name;
```

---

## 分析查询模板

### 模板13：同期群分析

```sql
-- 同期群留存分析
WITH cohorts AS (
    SELECT
        user_id,
        DATE(MIN(order_date)) as cohort_date,
        DATEDIFF(day, DATE(MIN(order_date)), DATE(order_date)) as days_since_first
    FROM orders
    GROUP BY user_id, DATE(MIN(order_date)), DATE(order_date)
),
cohort_sizes AS (
    SELECT
        cohort_date,
        COUNT(DISTINCT user_id) as cohort_size
    FROM cohorts
    WHERE days_since_first = 0
    GROUP BY cohort_date
)
SELECT
    c.cohort_date,
    cs.cohort_size,
    c.days_since_first,
    COUNT(DISTINCT c.user_id) as retained_users,
    ROUND(COUNT(DISTINCT c.user_id) * 100.0 / cs.cohort_size, 2) as retention_rate
FROM cohorts c
JOIN cohort_sizes cs ON c.cohort_date = cs.cohort_date
WHERE c.days_since_first IN (0, 1, 7, 30)
GROUP BY c.cohort_date, cs.cohort_size, c.days_since_first
ORDER BY c.cohort_date, c.days_since_first;
```

### 模板14：RFM分析

```sql
-- RFM分析
WITH user_rfm AS (
    SELECT
        user_id,
        DATEDIFF(CURDATE(), MAX(order_date)) as recency,
        COUNT(*) as frequency,
        SUM(amount) as monetary
    FROM orders
    WHERE status = 'completed'
    GROUP BY user_id
)
SELECT
    user_id,
    NTILE(4) OVER (ORDER BY recency DESC) as R_score,
    NTILE(4) OVER (ORDER BY frequency DESC) as F_score,
    NTILE(4) OVER (ORDER BY monetary DESC) as M_score,
    R_score + F_score + M_score as RFM_sum
FROM user_rfm
ORDER BY RFM_sum DESC;
```

### 模板15：漏斗分析

```sql
-- 漏斗分析
SELECT
    '注册' as stage,
    COUNT(DISTINCT user_id) as users
FROM users
UNION ALL
SELECT
    '首单' as stage,
    COUNT(DISTINCT user_id) as users
FROM orders
WHERE days_since_first = 0
UNION ALL
SELECT
    '复购' as stage,
    COUNT(DISTINCT user_id) as users
FROM orders
WHERE days_since_first > 0
ORDER BY stage;
```

---

## 性能优化模板

### 优化1：索引使用

```sql
-- 创建索引
CREATE INDEX idx_{table}_{column} ON {table_name}({column});

-- 查看索引
SHOW INDEX FROM {table_name};

-- 删除索引
DROP INDEX idx_{table}_{column} ON {table_name};
```

### 优化2：查询优化

```sql
-- 避免SELECT *
SELECT column1, column2 FROM table_name;

-- 使用LIMIT
SELECT * FROM table_name LIMIT 1000;

-- 使用EXPLAIN分析
EXPLAIN SELECT * FROM table_name WHERE condition;
```

---

## 数据清洗模板

### 模板16：缺失值处理

```sql
-- 检查缺失值
SELECT
    COUNT(*) as total_rows,
    COUNT(column1) as non_null_column1,
    COUNT(column2) as non_null_column2,
    COUNT(*) - COUNT(column1) as null_column1
FROM table_name;

-- 填充缺失值
UPDATE table_name
SET column1 = COALESCE(column1, {default_value});

-- 删除缺失值
DELETE FROM table_name
WHERE column1 IS NULL;
```

### 模板17：重复值处理

```sql
-- 查找重复值
SELECT
    column1,
    column2,
    COUNT(*) as count
FROM table_name
GROUP BY column1, column2
HAVING COUNT(*) > 1;

-- 删除重复值
DELETE t1 FROM table_name t1
JOIN table_name t2
WHERE t1.id > t2.id
  AND t1.column1 = t2.column1
  AND t1.column2 = t2.column2;
```

### 模板18：异常值处理

```sql
-- 识别异常值（使用标准差）
SELECT *
FROM (
    SELECT
        column1,
        AVG(column1) OVER () as avg_value,
        STDDEV(column1) OVER () as std_value
    FROM table_name
) t
WHERE ABS(column1 - avg_value) > 3 * std_value;

-- 处理异常值
UPDATE table_name
SET column1 = CASE
    WHEN ABS(column1 - avg_value) > 3 * std_value THEN avg_value
    ELSE column1
END;
```

---

## 最佳实践

### 1. 查询编写原则

- **明确需求** - 理解业务需求
- **逐步构建** - 从简单到复杂
- **测试验证** - 确认结果正确
- **性能优化** - 分析执行计划

### 2. 命名规范

- **表名：** 小写+下划线 (orders, order_items)
- **列名：** 小写+下划线 (user_id, order_date)
- **别名：** 驼峰命名 (orderCount, avgAmount)

### 3. 注释规范

```sql
-- 单行注释
/*
多行注释
*/
```

---

**模板版本：** v1.0
**最后更新：** 2026-03-20
