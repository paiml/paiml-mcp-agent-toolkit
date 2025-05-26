#!/bin/bash

echo "Setting up LocalStack for S3 testing..."

# Check if LocalStack is already running
if docker ps | grep -q localstack; then
    echo "LocalStack is already running"
else
    echo "Starting LocalStack..."
    docker run -d \
        --name localstack \
        -p 4566:4566 \
        -e SERVICES=s3 \
        -e DEFAULT_REGION=us-east-1 \
        localstack/localstack
    
    echo "Waiting for LocalStack to be ready..."
    sleep 10
fi

# Configure AWS CLI for LocalStack
export AWS_ENDPOINT_URL=http://localhost:4566
export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_DEFAULT_REGION=us-east-1

echo "Creating S3 bucket..."
aws --endpoint-url=$AWS_ENDPOINT_URL s3 mb s3://test-bucket 2>/dev/null || echo "Bucket already exists"

echo "Creating template directories..."
mkdir -p test_templates/upload

# Create metadata.json
cat > test_templates/upload/metadata.json << 'EOF'
{
  "uri": "template://makefile/rust/cli-binary",
  "name": "Rust CLI Makefile",
  "description": "Standard Makefile for Rust CLI applications",
  "toolchain": {
    "type": "rust",
    "cargo_features": ["clippy", "rustfmt"]
  },
  "category": "makefile",
  "parameters": [
    {
      "name": "project_name",
      "description": "Name of the Rust project",
      "param_type": "project_name",
      "required": true,
      "default_value": null,
      "validation_pattern": "^[a-z][a-z0-9_]*$"
    },
    {
      "name": "has_tests",
      "description": "Include test targets",
      "param_type": "boolean",
      "required": false,
      "default_value": "true",
      "validation_pattern": null
    },
    {
      "name": "has_benchmarks",
      "description": "Include benchmark targets",
      "param_type": "boolean",
      "required": false,
      "default_value": "false",
      "validation_pattern": null
    }
  ],
  "s3_object_key": "templates/makefile/rust/cli-binary/template.hbs",
  "content_hash": "dummy_hash",
  "semantic_version": "1.0.0",
  "dependency_graph": []
}
EOF

# Copy template file
cp test_templates/makefile-rust-cli-binary.hbs test_templates/upload/template.hbs

echo "Uploading templates to S3..."
aws --endpoint-url=$AWS_ENDPOINT_URL s3 cp test_templates/upload/metadata.json s3://test-bucket/templates/makefile/rust/cli-binary/metadata.json
aws --endpoint-url=$AWS_ENDPOINT_URL s3 cp test_templates/upload/template.hbs s3://test-bucket/templates/makefile/rust/cli-binary/template.hbs

echo "Verifying uploads..."
aws --endpoint-url=$AWS_ENDPOINT_URL s3 ls s3://test-bucket/templates/makefile/rust/cli-binary/

echo ""
echo "LocalStack S3 setup complete!"
echo ""
echo "To run the server with LocalStack:"
echo "export AWS_ENDPOINT_URL=http://localhost:4566"
echo "export TEMPLATES_BUCKET=test-bucket"
echo "export AWS_ACCESS_KEY_ID=test"
echo "export AWS_SECRET_ACCESS_KEY=test"
echo "cargo run --bin test_local"