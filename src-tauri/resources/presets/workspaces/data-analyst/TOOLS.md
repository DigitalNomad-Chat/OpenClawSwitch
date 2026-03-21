# TOOLS.md - 工具和资源清单

_数据分析师的推荐工具和资源库。_

---

## 推荐工具

### SQL工具

**数据库管理：**
- MySQL Workbench - MySQL数据库管理
- DBeaver - 通用数据库工具
- DataGrip - JetBrains数据库IDE
- phpMyAdmin - Web版MySQL管理

**SQL学习：**
- W3Schools SQL教程
- LeetCode Database题目
- SQLZoo - 交互式SQL学习

### Excel工具

**基础功能：**
- 数据透视表 - 数据分析
- 图表制作 - 数据可视化
- 函数库 - 数据计算
- 条件格式 - 数据突出

**高级功能：**
- Power Query - 数据清洗和转换
- Power Pivot - 数据建模
- Solver - 优化求解
- 宏 - 自动化

**学习资源：**
- Microsoft官方教程
- Excel Easy - 视频教程
- Excel Campus - 在线课程

### Python工具

**核心库：**
- Pandas - 数据处理和分析
- NumPy - 数值计算
- Matplotlib - 数据可视化
- Seaborn - 统计可视化
- Scikit-learn - 机器学习

**开发环境：**
- Jupyter Notebook - 交互式开发
- Google Colab - 云端Notebook
- Anaconda - Python发行版

**学习资源：**
- Python官方文档
- Pandas官方文档
- Kaggle Learn - 免费课程

### R工具

**核心包：**
- dplyr - 数据处理
- ggplot2 - 数据可视化
- tidyr - 数据整理
- readr - 数据导入
- shiny - 交互式应用

**开发环境：**
- RStudio - R语言IDE
- RStudio Cloud - 云端IDE

**学习资源：**
- R官方文档
- R for Data Science - 在线书籍
- Tidyverse Skills - 学习资源

### BI工具

**Tableau：**
- Tableau Desktop - 桌面版
- Tableau Public - 免费版
- Tableau Server - 企业版

**Power BI：**
- Power BI Desktop - 免费桌面版
- Power BI Service - 云服务
- Power BI Report Server - 企业版

**其他BI工具：**
- Qlik Sense
- Looker
- Sisense

---

## 数据可视化

### 图表类型

**比较类：**
- 柱状图 - 比较不同类别
- 条形图 - 比较排名
- 雷达图 - 多维度比较

**趋势类：**
- 折线图 - 时间趋势
- 面积图 - 累积趋势
- 散点图 - 相关性

**占比类：**
- 饼图 - 部分占整体
- 环形图 - 部分占整体（现代版）
- 树状图 - 层级占比
- 瀑堆图 - 复杂占比

**分布类：**
- 直方图 - 数值分布
- 箱线图 - 统计分布
- 小提琴图 - 密度分布

**关系类：**
- 散点图 - 相关性
- 气泡图 - 多维关系
- 热力图 - 矩阵关系

**地理类：**
- 地图 - 地理分布
- 填充地图 - 区域数据
- 符号地图 - 点数据

### 可视化原则

**颜色使用：**
- 使用对比色突出重点
- 保持颜色一致性
- 考虑色盲友好
- 避免过多颜色

**布局设计：**
- 保持简洁清晰
- 突出关键信息
- 避免视觉混乱
- 引导视线流动

**文字标注：**
- 标题简洁明了
- 标签清晰易读
- 注释提供背景
- 单位明确标注

---

## SQL常用语句

### 基础查询

```sql
-- 查询所有数据
SELECT * FROM table_name;

-- 查询指定列
SELECT column1, column2 FROM table_name;

-- 条件查询
SELECT * FROM table_name WHERE condition;

-- 排序查询
SELECT * FROM table_name ORDER BY column_name DESC;
```

### 聚合查询

```sql
-- 计数
SELECT COUNT(*) FROM table_name;

-- 求和
SELECT SUM(column_name) FROM table_name;

-- 平均值
SELECT AVG(column_name) FROM table_name;

-- 分组聚合
SELECT category, COUNT(*), SUM(amount)
FROM table_name
GROUP BY category;
```

### 连接查询

```sql
-- 内连接
SELECT a.*, b.column
FROM table_a a
JOIN table_b b ON a.id = b.id;

-- 左连接
SELECT a.*, b.column
FROM table_a a
LEFT JOIN table_b b ON a.id = b.id;
```

### 高级查询

```sql
-- 子查询
SELECT * FROM table_name
WHERE id IN (SELECT id FROM other_table);

-- 窗口函数
SELECT
    column1,
    column2,
    ROW_NUMBER() OVER (PARTITION BY column1 ORDER BY column2) as rn
FROM table_name;

-- CASE语句
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

## Excel常用函数

### 基础函数

```excel
=SUM(range)           -- 求和
=AVERAGE(range)       -- 平均值
=COUNT(range)         -- 计数
=MAX(range)           -- 最大值
=MIN(range)           -- 最小值
```

### 逻辑函数

```excel
=IF(condition, value_if_true, value_if_false)  -- 条件判断
=AND(condition1, condition2)                     -- 逻辑与
=OR(condition1, condition2)                      -- 逻辑或
=NOT(condition)                                 -- 逻辑非
```

### 查找函数

```excel
=VLOOKUP(lookup_value, table_array, col_index, range_lookup)  -- 垂直查找
=HLOOKUP(lookup_value, table_array, row_index, range_lookup)  -- 水平查找
=INDEX(array, row_num, column_num)                         -- 索引查找
=MATCH(lookup_value, lookup_array, match_type)               -- 匹配查找
```

### 文本函数

```excel
=CONCAT(text1, text2, ...)  -- 连接文本
=LEFT(text, num_chars)      -- 左侧字符
=RIGHT(text, num_chars)     -- 右侧字符
=MID(text, start_num, num_chars)  -- 中间字符
=TRIM(text)               -- 去除空格
```

### 日期函数

```excel
=TODAY()                  -- 今天日期
=NOW()                   -- 当前日期时间
=YEAR(date)               -- 年份
=MONTH(date)              -- 月份
=DAY(date)                -- 日
=DATEDIF(start_date, end_date, unit)  -- 日期差
```

---

## Python常用代码

### Pandas基础

```python
import pandas as pd

# 读取数据
df = pd.read_csv('file.csv')
df = pd.read_excel('file.xlsx')

# 查看数据
df.head()           # 前几行
df.info()           # 数据信息
df.describe()       # 描述统计

# 数据筛选
df[df['column'] > value]
df[(df['col1'] > 0) & (df['col2'] < 10)]

# 数据聚合
df.groupby('column').agg({'col1': 'sum', 'col2': 'mean'})
```

### 数据可视化

```python
import matplotlib.pyplot as plt
import seaborn as sns

# 折线图
plt.plot(df['x'], df['y'])
plt.show()

# 柱状图
plt.bar(df['category'], df['value'])
plt.show()

# 散点图
plt.scatter(df['x'], df['y'])
plt.show()

# 热力图
sns.heatmap(df.corr())
plt.show()
```

---

## 统计学基础

### 描述性统计

**集中趋势：**
- 均值（Mean）：平均值
- 中位数（Median）：中间值
- 众数（Mode）：出现最多的值

**离散程度：**
- 方差（Variance）：数据分散程度
- 标准差（Standard Deviation）：方差的平方根
- 极差（Range）：最大值与最小值的差

**分布形状：**
- 偏度（Skewness）：分布对称性
- 峰度（Kurtosis）：分布尖峭程度

### 推断性统计

**假设检验：**
- t检验：比较两组均值
- 卡方检验：检验分类变量关联
- 方差分析（ANOVA）：比较多组均值

**相关分析：**
- 相关系数：变量间关系强度
- 回归分析：变量间因果关系

---

## 外部资源

### 学习平台

**在线课程：**
- Coursera - 数据分析课程
- edX - 统计学课程
- Udemy - 实战课程
- Kaggle Learn - 免费实战

**书籍推荐：**
- 《深入浅出数据分析》
- 《Python数据分析实战》
- 《SQL必知必会》
- 《数据可视化之美》

**社区资源：**
- Kaggle - 数据科学竞赛
- Stack Overflow - 技术问答
- Reddit r/dataanalysis - 数据分析社区
- 知乎数据分析话题

---

## 最佳实践

### 数据管理
- 定期备份数据
- 建立数据字典
- 记录数据处理过程
- 版本控制分析代码

### 分析流程
- 明确分析目标
- 验证数据质量
- 使用多种方法验证
- 记录分析过程

### 报告撰写
- 从结论开始
- 用可视化支持
- 提供可行建议
- 保持客观中立

---

_最后更新：2026-03-20 | 版本：v1.0_
