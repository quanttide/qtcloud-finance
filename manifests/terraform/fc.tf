# FC 默认角色：允许 FC 服务挂载弹性网卡访问 VPC（应用级）
resource "alicloud_ram_role" "fc" {
  role_name                   = "${local.app_name_prefix}-fc"
  assume_role_policy_document = jsonencode({
    Statement = [{
      Action    = "sts:AssumeRole"
      Effect    = "Allow"
      Principal = { Service = ["fc.aliyuncs.com"] }
    }]
    Version = "1"
  })
  description = "Function Compute 默认角色（qtcloud-finance）"
}

resource "alicloud_ram_role_policy_attachment" "fc_vpc" {
  policy_name = "AliyunECSNetworkInterfaceManagementAccess"
  policy_type = "System"
  role_name   = alicloud_ram_role.fc.role_name
}

# 函数计算（FC 3.0）：custom-container 容器镜像，内置 vibe-coding 课程数据
resource "alicloud_fcv3_function" "this" {
  function_name = local.app_name_prefix
  description   = "qtcloud-finance 财务预算 API"
  runtime       = "custom-container"
  handler       = "index.handler"
  cpu           = 0.5
  memory_size   = var.fc_memory
  disk_size     = 512
  timeout       = var.fc_timeout
  internet_access = true
  role          = alicloud_ram_role.fc.arn

  custom_container_config {
    image = var.image
    port  = 8080
  }

  tags = {
    project     = var.project
    environment = var.environment
  }
}

# HTTP 触发器：直接访问（后续经 API 网关统一接入）
resource "alicloud_fcv3_trigger" "http" {
  function_name = alicloud_fcv3_function.this.function_name
  trigger_name  = "http"
  trigger_type  = "http"
  qualifier     = "LATEST"
  trigger_config = jsonencode({
    authType = "anonymous"
    methods  = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS"]
  })
}
