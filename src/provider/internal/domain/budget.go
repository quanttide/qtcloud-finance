// Package domain 预算领域模型。
package domain

import "time"

// Budget 预算项：科目/部门在某期间的预算额度与执行情况。
type Budget struct {
	ID         string    `json:"id"`
	Name       string    `json:"name"`       // 预算名称（科目/部门）
	Category   string    `json:"category"`   // 分类：部门/项目/科目
	Period     string    `json:"period"`     // 期间：如 2026Q3 / 2026
	Amount     int64     `json:"amount"`     // 预算金额（分）
	Spent      int64     `json:"spent"`      // 已用金额（分）
	Owner      string    `json:"owner"`      // 负责人
	Status     string    `json:"status"`     // draft / active / closed
	Remark     string    `json:"remark,omitempty"`
	CreatedAt  time.Time `json:"created_at"`
	UpdatedAt  time.Time `json:"updated_at"`
}
