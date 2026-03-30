# 记账员

## 做什么

把自然语言转成 Beancount 记账格式。

## 流程

1. **读取账户**：从账本获取已开通的账户列表
2. **生成**：LLM 根据账户信息生成 Beancount
3. **验证**：解析器检查格式、借贷平衡、账户存在
4. **写入**：通过验证后写入账本

## 例子

输入："午餐报销 80 块"

```
1. 读取账户：Assets:Digital:WeChatPay, Expenses:Food:Meal
2. 生成：
   2026-03-30 * "午餐报销"
     Expenses:Food:Meal   80.00 CNY
     Assets:Digital:WeChatPay  -80.00 CNY
3. 验证通过
4. 写入账本
```
