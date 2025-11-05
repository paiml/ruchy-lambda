#!/bin/bash
# Build C++ baseline Lambda function
# Source: lambda-perf cpp11_on_provided_al2023

set -euo pipefail

echo "🔨 Building C++ baseline Lambda..."
echo "⚠️  Requires AWS Lambda C++ SDK"

# This requires Docker to build with proper dependencies
# See https://github.com/awslabs/aws-lambda-cpp for full build instructions

echo "Using Docker to build C++ Lambda..."

# Create Dockerfile if not exists
cat > Dockerfile <<'EOF'
FROM amazonlinux:2023

RUN yum install -y gcc-c++ make cmake3 zip git libcurl-devel libstdc++-static

# Clone and build AWS Lambda C++ SDK
RUN git clone https://github.com/awslabs/aws-lambda-cpp.git /tmp/lambda-cpp && \
    cd /tmp/lambda-cpp && \
    mkdir build && cd build && \
    cmake3 .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/tmp/install && \
    make && make install

# Copy source code
COPY lambda/main.cpp /tmp/main.cpp

# Build the function
RUN cd /tmp && \
    g++ -std=c++11 main.cpp -o bootstrap \
        -I/tmp/install/include \
        -L/tmp/install/lib64 \
        -laws-lambda-runtime \
        -lcurl \
        -static-libstdc++ && \
    zip function.zip bootstrap

CMD ["cat", "/tmp/function.zip"]
EOF

# Build with Docker
docker build -t baseline-cpp .
docker create --name baseline-cpp-container baseline-cpp
docker cp baseline-cpp-container:/tmp/function.zip function.zip
docker rm baseline-cpp-container

echo "✅ C++ baseline built: function.zip"
echo "Binary size: $(ls -lh function.zip | awk '{print $5}')"
