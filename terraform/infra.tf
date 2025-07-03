# ========================================
# suzaku ECS Fargate
# VPC create to task run
# ========================================

provider "aws" {
  region = "ap-northeast-1"
}

# -------------------------------
# VPC
# -------------------------------
resource "aws_vpc" "suzaku_vpc" {
  cidr_block = "10.0.0.0/16"
  enable_dns_support   = true
  enable_dns_hostnames = true
  tags = {
    Name = "suzaku-vpc"
  }
}

# Internet Gateway
resource "aws_internet_gateway" "suzaku_igw" {
  vpc_id = aws_vpc.suzaku_vpc.id
  tags = {
    Name = "suzaku-igw"
  }
}

# Public Subnet
resource "aws_subnet" "suzaku_public_subnet" {
  vpc_id                  = aws_vpc.suzaku_vpc.id
  cidr_block              = "10.0.1.0/24"
  availability_zone       = "ap-northeast-1a"
  map_public_ip_on_launch = true
  tags = {
    Name = "suzaku-public-subnet"
  }
}

# Route Table
resource "aws_route_table" "suzaku_public_rt" {
  vpc_id = aws_vpc.suzaku_vpc.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.suzaku_igw.id
  }
  tags = {
    Name = "suzaku-public-rt"
  }
}

# Route Table Association
resource "aws_route_table_association" "suzaku_public_rt_assoc" {
  subnet_id      = aws_subnet.suzaku_public_subnet.id
  route_table_id = aws_route_table.suzaku_public_rt.id
}

# -------------------------------
# Security Group
# -------------------------------
resource "aws_security_group" "suzaku_sg" {
  name        = "suzaku-sg"
  description = "Allow all outbound traffic"
  vpc_id      = aws_vpc.suzaku_vpc.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# -------------------------------
# IAM Role for ECS Task
# -------------------------------
resource "aws_iam_role" "suzaku_task_role" {
  name = "suzaku-task-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        },
        Action = "sts:AssumeRole"
      }
    ]
  })
}

# IAM Policy for S3 and SSM Messages
resource "aws_iam_role_policy" "suzaku_task_policy" {
  name = "suzaku-task-policy"
  role = aws_iam_role.suzaku_task_role.id

  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Action = ["s3:GetObject"],
        Resource = var.suzaku_task_policy_resources["get_object"]
      },
      {
        Effect = "Allow",
        Action = ["s3:ListBucket"],
        Resource = var.suzaku_task_policy_resources["list_bucket"]
      },
      {
        Effect = "Allow",
        Action = ["s3:PutObject"],
        Resource = var.suzaku_task_policy_resources["put_object"]
      },
      {
        "Effect": "Allow",
        "Action": [
            "logs:CreateLogStream",
            "logs:PutLogEvents"
        ],
        "Resource": "arn:aws:logs:*:*:log-group:/ecs/suzaku:*"
      },
      {
        Effect = "Allow",
        Action = [
          "ssmmessages:CreateControlChannel",
          "ssmmessages:CreateDataChannel",
          "ssmmessages:OpenControlChannel",
          "ssmmessages:OpenDataChannel"
        ],
        Resource = "*"
      }
    ]
  })
}

# -------------------------------
# ECS Cluster
# -------------------------------
resource "aws_ecs_cluster" "suzaku_cluster" {
  name = "suzaku-ecs-cluster"
}

# -------------------------------
# ECS Task Definition
# -------------------------------
resource "aws_ecs_task_definition" "suzaku_task_def" {
  family                   = "suzaku-task"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "1024"   # 1 vCPU
  memory                   = "3072"   # 3GB
  execution_role_arn       = aws_iam_role.suzaku_task_role.arn
  task_role_arn            = aws_iam_role.suzaku_task_role.arn

  container_definitions = jsonencode([
    {
      name      = "suzaku"
      image     = "public.ecr.aws/q6q8s3z6/suzaku:latest"
      essential = true
      command   = ["/bin/bash", "-c", "while true; do sleep 3600; done"]
      logConfiguration = {
        logDriver = "awslogs",
        options = {
          awslogs-group         = "/ecs/suzaku"
          awslogs-region        = "ap-northeast-1"
          awslogs-stream-prefix = "ecs"
        }
      }
    }
  ])
}

# -------------------------------
# ECS Service
# -------------------------------
resource "aws_ecs_service" "suzaku_service" {
  name            = "suzaku-service"
  cluster         = aws_ecs_cluster.suzaku_cluster.id
  task_definition = aws_ecs_task_definition.suzaku_task_def.arn
  desired_count   = 1
  launch_type     = "FARGATE"
  enable_execute_command = true

  network_configuration {
    subnets         = [aws_subnet.suzaku_public_subnet.id]
    security_groups = [aws_security_group.suzaku_sg.id]
    assign_public_ip = true
  }
  depends_on = [aws_cloudwatch_log_group.suzaku_log_group]
}


# CloudWatch Logs Group
resource "aws_cloudwatch_log_group" "suzaku_log_group" {
  name              = "/ecs/suzaku"
  retention_in_days = 7
}

# -------------------------------
# Output Variables
# -------------------------------
output "vpc_id" {
  value = aws_vpc.suzaku_vpc.id
}

output "subnet_id" {
  value = aws_subnet.suzaku_public_subnet.id
}

output "security_group_id" {
  value = aws_security_group.suzaku_sg.id
}

output "ecs_cluster_name" {
  value = aws_ecs_cluster.suzaku_cluster.name
}

output "task_definition_arn" {
  value = aws_ecs_task_definition.suzaku_task_def.arn
}
