# Run Suzaku on AWS ECS Fargate

This repository provides the Terraform configuration and execution steps to run [Suzaku](https://github.com/Yamato-Security/suzaku), a digital forensics and threat detection tool, on AWS ECS Fargate. The flow includes setting up infrastructure, running Suzaku against CloudTrail logs stored in S3, and tearing down the infrastructure afterwards.

---

## Prerequisites

- AWS CLI configured (`aws configure`)
- Terraform installed (`>= v1.0`)
- An S3 bucket with CloudTrail logs uploaded
- Permissions to create ECS resources, IAM roles, VPC, etc.

---

## Configuration: `terraform.tfvars`
Before running Terraform, create a `terraform.tfvars` file in the terraform directory with the following content:

```hcl
suzaku_task_policy_resources = {
  get_object      = "arn:aws:s3:::suzaku-input-logs/*"
  list_bucket     = "arn:aws:s3:::suzaku-input-logs"
  put_object      = "arn:aws:s3:::suzaku-output-logs/*"
  cloudwatch_logs = "arn:aws:logs:*:*:log-group:/ecs/suzaku:*"
}
```

### Explanation
- get_object: Grants read access to CloudTrail logs stored in the specified S3 bucket.
- list_bucket: Allows listing the contents of the input bucket.
- put_object: Allows writing output files (e.g., reports) to another S3 location.
- cloudwatch_logs: Grants permission to write logs to CloudWatch for debugging or monitoring purposes.

Replace the S3 bucket names with your actual input/output bucket names.


## Step-by-step Execution

### 1. Initialize Terraform

Initialize the Terraform working directory to download providers and modules.

```bash
terraform init
```

### 2. Plan Infrastructure Changes
Review the execution plan to understand what will be created.

```bash
terraform plan
```

### 3. Apply Infrastructure
Provision all the required AWS resources using Terraform.

```bash
terraform apply -auto-approve
```

This sets up:
- ECS cluster and task definition
- VPC, subnet, and security group
- IAM roles and execution policies
- Log group for CloudWatch


### 4. Run ECS Task
Manually start an ECS Fargate task to run suzaku.
```bash
aws ecs run-task \
  --cluster suzaku-ecs-cluster \
  --launch-type FARGATE \
  --network-configuration 'awsvpcConfiguration={subnets=[${subnet_id}],securityGroups=[${security_group_id}],assignPublicIp="ENABLED"}' \
  --task-definition suzaku-task \
  --enable-execute-command
```

💡 Note: Replace ${subnet_id} and ${security_group_id} with actual values from Terraform output.

### 5. Log in to the ECS Container
Wait a few moments for the task to start, then connect to it using execute-command.
```bash
aws ecs execute-command \
  --cluster suzaku-ecs-cluster \
  --task ${task_id} \
  --container suzaku \
  --interactive \
  --command "/bin/bash"
```
💡 Note: Replace ${task_id} with the ID returned from the run-task command.

### 6. Copy CloudTrail Logs from S3
Download CloudTrail log files from your S3 bucket to the ECS container.

```bash
mkdir /tmp/s3
aws s3 cp ${s3_path} /tmp/s3/ --recursive
```
💡 Note: Replace ${s3_path} with your actual S3 path, e.g., s3://your-bucket/path/.

### 7. Run Suzaku
Execute the Suzaku CLI commands within the container.
```bash
suzaku update-rules
suzaku aws-ct-metrics --directory /tmp/s3/
```

### 8. Stop the ECS Task
After analysis, stop the running ECS task.
```bash
aws ecs stop-task \
  --cluster suzaku-ecs-cluster \
  --task ${task_id}
```

### 9. Destroy Infrastructure
Tear down all the provisioned resources.
```bash
terraform destroy -auto-approve
```

