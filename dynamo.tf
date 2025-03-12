# dynamo.tf
resource "aws_dynamodb_table" "test-table" {
    name          = "users"
    billing_mode  = "PAY_PER_REQUEST"
    read_capacity  = 0
    write_capacity = 0

    attribute {
        name = "username"
        type = "S"
    }

}