variable "region" {
  description = "阿里云地域"
  type        = string
  default     = "cn-hangzhou"
}

variable "project" {
  description = "项目名（资源命名前缀）"
  type        = string
  default     = "qtcloud-finance"
}

variable "environment" {
  description = "环境：dev / prod"
  type        = string
  default     = "prod"
}

variable "image" {
  description = "FC 容器镜像（ACR 地址）。由 CI 注入"
  type        = string
}

variable "fc_memory" {
  description = "FC 函数内存（MB）"
  type        = number
  default     = 512
}

variable "fc_timeout" {
  description = "FC 函数超时（秒）"
  type        = number
  default     = 60
}
