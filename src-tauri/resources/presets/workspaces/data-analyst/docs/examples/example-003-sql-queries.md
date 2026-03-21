# 案例 003: SQL数据查询实战

**业务场景：** 电商数据分析
**查询目标：** 从数据库提取和分析销售数据
**数据库类型：** MySQL

---

## 基础查询

### 查询1：基础数据查询

**需求：** 查询订单表的基本信息

```sql
-- 查询所有订单
SELECT
    order_id,
    user_id,
    order_date,
    amount,
    status
FROM orders
LIMIT 100;
```

**查询结果：**
```
order_id | user_id | order_date | amount | status
---------|---------|------------|--------|--------
ORD001   | U001    | 2026-01-01 | 299    | completed
ORD002   | U002    | 2026-01-01 | 199    | completed
ORD003   | U003    | 2026-01-02 | 599    | pending
```

### 查询2：条件查询

**需求：** 查询2026年1月的已完成订单

```sql
SELECT
    order_id,
    user_id,
    order_date,
    amount
FROM orders
WHERE order_date >= '2026-01-01'
  AND order_date < '2026-02-01'
  AND status = 'completed'
ORDER BY order_date DESC;
```

### 查询3：聚合查询

**需求：** 统计每日销售额和订单数

```sql
SELECT
    DATE(order_date) as order_date,
    COUNT(*) as order_count,
    SUM(amount) as total_amount,
    AVG(amount) as avg_amount
FROM orders
WHERE status = 'completed'
GROUP BY DATE(order_date)
ORDER BY order_date;
```

**查询结果：**
```
order_date | order_count | total_amount | avg_amount
-----------|-------------|--------------|------------
2026-01-01 | 1,250        | 250,000      | 200
2026-01-02 | 1,180        | 236,000      | 200
2026-01-03 | 1,320        | 264,000      | 200
```

---

## 高级查询

### 查询4：多表连接

**需求：** 查询订单及其用户信息

```sql
SELECT
    o.order_id,
    o.order_date,
    o.amount,
    u.user_name,
    u.email,
    u.register_date
FROM orders o
JOIN users u ON o.user_id = u.user_id
WHERE o.status = 'completed'
ORDER BY o.order_date DESC;
```

### 查询5：子查询

**需求：** 查询购买次数最多的前10名用户

```sql
SELECT
    u.user_id,
    u.user_name,
    COUNT(o.order_id) as order_count,
    SUM(o.amount) as total_amount
FROM users u
JOIN orders o ON u.user_id = o.user_id
GROUP BY u.user_id, u.user_name
ORDER BY order_count DESC
LIMIT 10;
```

**查询结果：**
```
user_id | user_name | order_count | total_amount
---------|-----------|-------------|--------------
U001234 | 张三      | 45          | 9,000
U002345 | 李四      | 38          | 7,600
U003456 | 王五      | 32          | 6,400
```

### 查询6：窗口函数

**需求：** 计算每个订单的排名

```sql
SELECT
    order_id,
    user_id,
    order_date,
    amount,
    ROW_NUMBER() OVER (
        PARTITION BY user_id
        ORDER BY order_date DESC
    ) as order_rank
FROM orders
WHERE status = 'completed';
```

### 查询7：CASE表达式

**需求：** 对订单金额进行分类

```sql
SELECT
    order_id,
    amount,
    CASE
        WHEN amount < 100 THEN '小额'
        WHEN amount >= 100 AND amount < 500 THEN '中额'
        WHEN amount >= 500 AND amount < 1000 THEN '大额'
        ELSE '超大额'
    END as amount_category
FROM orders;
```

---

## 数据分析查询

### 分析1：用户RFM分析

**需求：** 计算用户的RFM值

```sql
-- RFM分析：Recency, Frequency, Monetary
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
    (recency + frequency + monetary) as RFM_sum
FROM user_rfm;
```

### 分析2：同期群分析

**需求：** 计算用户留存率

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
GROUP BY c.cohort_date, cs.cohort_size, c.days_since_first
ORDER BY c.cohort_date, c.days_since_first;
```

**查询结果：**
```
cohort_date | cohort_size | days_since_first | retained_users | retention_rate
------------|-------------|------------------|----------------|---------------
2026-01-01  | 10,000      | 0                | 10,000         | 100.00
2026-01-01  | 10,000      | 1                | 4,500          | 45.00
2026-01-01  | 10,000      | 7                | 2,500          | 25.00
2026-01-01  | 10,000      | 30               | 1,500          | 15.00
```

---

## 优化查询

### 优化1：索引优化

**问题查询：**
```sql
SELECT * FROM orders
WHERE user_id = 'U001234'
  AND order_date >= '2026-01-01';
```

**优化方案：**
```sql
-- 创建索引
CREATE INDEX idx_user_date ON orders(user_id, order_date);

-- 查询自动使用索引
EXPLAIN SELECT * FROM orders
WHERE user_id = 'U001234'
  AND order_date >= '2026-01-01';
```

### 优化2：查询重构

**问题查询：**
```sql
-- 子查询效率低
SELECT
    user_id,
    user_name,
    (SELECT COUNT(*) FROM orders WHERE user_id = u.user_id) as order_count
FROM users u;
```

**优化方案：**
```sql
-- 使用JOIN替代子查询
SELECT
    u.user_id,
    u.user_name,
    COUNT(o.order_id) as order_count
FROM users u
LEFT JOIN orders o ON u.user_id = o.user_id
GROUP BY u.user_id, u.user_name;
```

---

## 实用技巧

### 技巧1：数据处理

```sql
-- 去重
SELECT DISTINCT user_id FROM orders;

-- 限制返回行数
SELECT * FROM orders LIMIT 100;

-- 数据类型转换
SELECT CAST(amount AS DECIMAL(10,2)) FROM orders;
```

### 技巧2：字符串处理

```sql
-- 字符串拼接
SELECT CONCAT(user_name, '@', email_domain) as email
FROM users;

-- 字符串截取
SELECT SUBSTRING(email, 1, LOCATE('@', email) - 1) as username
FROM users;

-- 字符串替换
SELECT REPLACE(phone_number, '-', '') as clean_phone
FROM users;
```

### 技巧3：日期处理

```sql
-- 日期格式化
SELECT DATE_FORMAT(order_date, '%Y-%m-%d') as formatted_date
FROM orders;

-- 日期计算
SELECT
    order_date,
    DATE_ADD(order_date, INTERVAL 7 DAY) as plus_7days,
    DATEDIFF(NOW(), order_date) as days_ago
FROM orders;

-- 提取日期部分
SELECT
    DATE(order_date) as date_part,
    TIME(order_date) as time_part
FROM orders;
```

---

## 最佳实践

### 1. 查询编写原则

- **明确需求：** 先理解业务需求
- **逐步构建：** 从简单到复杂
- **测试验证：** 确认查询结果正确
- **优化性能：** 分析查询执行计划

### 2. 性能优化

- **使用索引：** 为常用查询字段创建索引
- **避免SELECT *：** 只查询需要的字段
- **限制结果：** 使用LIMIT限制返回行数
- **优化JOIN：** 小表驱动大表

### 3. 可读性

- **格式化代码：** 使用缩进和换行
- **添加注释：** 说明查询目的
- **命名规范：** 使用有意义的别名
- **分阶段测试：** 复杂查询分步测试

---

**分析完成时间：** 2026年4月12日
**分析师：** 数据喵
**案例版本：** v1.0
