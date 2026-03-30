# 记账员

## 概述

把自然语言描述转成 Beancount 记账格式。让大模型做打字员，让解析器做审核员。

## 场景

**场景一：员工报销**

输入：员工说"今天中午出差午餐花了 80 块"

输出：
```
2026-03-30 * "午餐报销"
  Expenses:Travel:Meal   80.00 CNY
  Assets:Digital:WeChatPay  -80.00 CNY
```

**场景二：发工资**

输入："3月工资 15000 转到建行卡"

输出：
```
2026-03-30 * "3月工资"
  Expenses:Salary:Wages   15000.00 CNY
  Assets:Bank:ChinaConstructionBank  -15000.00 CNY
```

## 验证

- 语法：Beancount 解析器检查格式
- 借贷：正负金额必须相等
- 账户：必须是账本里已开通的账户
