# NEX2426 Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying NEX2426 in various environments, from development setups to production deployments.

## Table of Contents

- [System Requirements](#system-requirements)
- [Installation Methods](#installation-methods)
- [Configuration](#configuration)
- [Production Deployment](#production-deployment)
- [Docker Deployment](#docker-deployment)
- [Cloud Deployment](#cloud-deployment)
- [Monitoring and Maintenance](#monitoring-and-maintenance)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)

---

## System Requirements

### Minimum Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | x86_64, 2 cores | x86_64, 4+ cores, AVX2 |
| **Memory** | 2 GB RAM | 8 GB RAM |
| **Storage** | 100 MB free | 1 GB free |
| **OS** | Linux (Ubuntu 20.04+) | Linux (Ubuntu 22.04+) |
| **Rust** | 1.85+ | 1.85+ |

### Performance Requirements

| Use Case | CPU Cores | Memory | Cost Factor |
|----------|------------|--------|-------------|
| **Development** | 2 | 4 GB | 1-3 |
| **Production** | 4+ | 8 GB+ | 3-5 |
| **High-Performance** | 8+ | 16 GB+ | 5-10 |

### Network Requirements

- **Bandwidth**: 1 Mbps+ for file operations
- **Latency**: <100ms for interactive use
- **Firewall**: No special ports required

---

## Installation Methods

### Method 1: Automated Installation (Recommended)

```bash
# Download and run installation script
curl -fsSL https://raw.githubusercontent.com/nex2426/nex2426/main/install.sh | sudo bash

# Or download and run manually
wget https://raw.githubusercontent.com/nex2426/nex2426/main/install.sh
chmod +x install.sh
sudo ./install.sh
```

**Features:**
- Automatic dependency installation
- System integration
- Service setup
- Configuration creation
- Post-install testing

### Method 2: Manual Installation

```bash
# Clone repository
git clone https://github.com/nex2426/nex2426.git
cd nex2426

# Build from source
cargo build --release

# Install binary
sudo cp target/release/nex2426 /usr/local/bin/
sudo chmod +x /usr/local/bin/nex2426

# Install documentation
sudo mkdir -p /usr/local/share/doc/nex2426
sudo cp -r docs/* /usr/local/share/doc/nex2426/

# Install man page
sudo mkdir -p /usr/local/share/man/man1
sudo cp docs/nex2426.1 /usr/local/share/man/man1/
sudo mandb
```

### Method 3: Package Installation

#### Debian/Ubuntu

```bash
# Download .deb package
wget https://github.com/nex2426/nex2426/releases/latest/download/nex2426_0.1.0_amd64.deb

# Install package
sudo dpkg -i nex2426_0.1.0_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f
```

#### RHEL/CentOS

```bash
# Download .rpm package
wget https://github.com/nex2426/nex2426/releases/latest/download/nex2426-0.1.0-1.x86_64.rpm

# Install package
sudo rpm -i nex2426-0.1.0-1.x86_64.rpm
```

### Method 4: Cargo Install

```bash
# Install from crates.io
cargo install nex2426

# Or install from git repository
cargo install --git https://github.com/nex2426/nex2426.git
```

---

## Configuration

### Default Configuration File

Configuration is stored in `/etc/nex2426/config.toml`:

```toml
[default]
# Default cost factor (1-10)
cost = 3

# Enable temporal binding by default
temporal_binding = true

# Number of threads (0 = auto)
threads = 0

# Memory limit in MB (0 = unlimited)
memory_limit = 0

[security]
# Enable bio-lock by default for file encryption
bio_lock_default = false

# Enable stealth mode by default
stealth_default = false

# Minimum key length
min_key_length = 8

[performance]
# Enable AVX2 optimizations (if available)
avx2_optimization = true

# Enable parallel processing
parallel_processing = true

# Cache size in MB
cache_size = 64

[logging]
# Log level (error, warn, info, debug, trace)
level = "info"

# Log file path (empty = stderr)
file = "/var/log/nex2426/nex2426.log"

# Enable performance logging
performance_logging = false
```

### Environment Variables

```bash
# Override configuration values
export NEX2426_COST=5
export NEX2426_THREADS=8
export NEX2426_MEMORY_LIMIT=4096
export NEX2426_LOG_LEVEL=debug
export NEX2426_CONFIG_FILE=/path/to/custom/config.toml
```

### User Configuration

Users can create personal configuration:

```bash
# Create user config directory
mkdir -p ~/.config/nex2426

# Create user config file
cat > ~/.config/nex2426/config.toml << 'EOF'
[default]
cost = 2
temporal_binding = false

[security]
min_key_length = 12
EOF
```

---

## Production Deployment

### System Preparation

```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Install required dependencies
sudo apt install -y build-essential pkg-config libssl-dev

# Create dedicated user
sudo useradd -r -s /bin/false -d /var/lib/nex2426 nex2426

# Create directories
sudo mkdir -p /var/lib/nex2426
sudo mkdir -p /var/log/nex2426
sudo mkdir -p /etc/nex2426

# Set permissions
sudo chown -R nex2426:nex2426 /var/lib/nex2426
sudo chown -R nex2426:nex2426 /var/log/nex2426
sudo chmod 755 /etc/nex2426
```

### Service Configuration

Create systemd service:

```bash
# Create service file
sudo tee /etc/systemd/system/nex2426.service > /dev/null << 'EOF'
[Unit]
Description=NEX2426 Quantum-Resistant Encryption Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/nex2426 --service
Restart=always
RestartSec=5
User=nex2426
Group=nex2426

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/nex2426

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable nex2426
sudo systemctl start nex2426
```

### Performance Tuning

```bash
# Optimize kernel parameters
sudo tee -a /etc/sysctl.conf << 'EOF'
# NEX2426 performance tuning
vm.swappiness=10
net.core.rmem_max=134217728
net.core.wmem_max=134217728
EOF

# Apply kernel parameters
sudo sysctl -p

# Set CPU governor to performance
echo 'performance' | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### High Availability Setup

```bash
# Create load balancer configuration (nginx example)
sudo tee /etc/nginx/sites-available/nex2426 << 'EOF'
upstream nex2426_backend {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}

server {
    listen 80;
    server_name nex2426.example.com;
    
    location / {
        proxy_pass http://nex2426_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
EOF

# Enable site
sudo ln -s /etc/nginx/sites-available/nex2426 /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

---

## Docker Deployment

### Dockerfile

```dockerfile
# Multi-stage build for minimal image
FROM rust:1.85 as builder

WORKDIR /app
COPY . .

# Build optimized binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create user
RUN useradd -r -s /bin/false nex2426

# Copy binary and create directories
COPY --from=builder /app/target/release/nex2426 /usr/local/bin/
RUN mkdir -p /etc/nex2426 /var/lib/nex2426 /var/log/nex2426
RUN chown -R nex2426:nex2426 /var/lib/nex2426 /var/log/nex2426

# Set permissions
RUN chmod +x /usr/local/bin/nex2426

# Expose port (if running as service)
EXPOSE 8080

# Switch to non-root user
USER nex2426

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD nex2426 --version || exit 1

# Default command
CMD ["nex2426", "--service"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  nex2426:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./config:/etc/nex2426:ro
      - nex2426_data:/var/lib/nex2426
      - nex2426_logs:/var/log/nex2426
    environment:
      - NEX2426_COST=3
      - NEX2426_THREADS=4
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "nex2426", "--version"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped

volumes:
  nex2426_data:
  nex2426_logs:
  redis_data:
```

### Docker Commands

```bash
# Build image
docker build -t nex2426:latest .

# Run container
docker run -d \
  --name nex2426 \
  -p 8080:8080 \
  -v $(pwd)/config:/etc/nex2426:ro \
  -v nex2426_data:/var/lib/nex2426 \
  nex2426:latest

# Run with docker-compose
docker-compose up -d

# Check logs
docker logs -f nex2426

# Execute in container
docker exec -it nex2426 bash
```

---

## Cloud Deployment

### AWS Deployment

#### EC2 Instance

```bash
# Create EC2 instance with appropriate specs
aws ec2 run-instances \
  --image-id ami-0c02fb55956c7d316 \
  --instance-type c5.xlarge \
  --key-name my-key-pair \
  --security-group-ids sg-903004f8 \
  --subnet-id subnet-6e7f829e \
  --user-data file://user-data.sh \
  --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=nex2426-server}]'
```

#### User Data Script

```bash
#!/bin/bash
# user-data.sh

# Update system
apt-get update -y
apt-get upgrade -y

# Install dependencies
apt-get install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source /home/ubuntu/.cargo/env

# Clone and build NEX2426
git clone https://github.com/nex2426/nex2426.git /opt/nex2426
cd /opt/nex2426
cargo build --release

# Install binary
cp target/release/nex2426 /usr/local/bin/
chmod +x /usr/local/bin/nex2426

# Create service
useradd -r -s /bin/false nex2426
mkdir -p /var/lib/nex2426 /var/log/nex2426
chown -R nex2426:nex2426 /var/lib/nex2426 /var/log/nex2426

# Create systemd service
cat > /etc/systemd/system/nex2426.service << 'EOF'
[Unit]
Description=NEX2426 Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/nex2426 --service
User=nex2426
Restart=always

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable nex2426
systemctl start nex2426
```

### Kubernetes Deployment

#### Deployment YAML

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nex2426
  labels:
    app: nex2426
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nex2426
  template:
    metadata:
      labels:
        app: nex2426
    spec:
      containers:
      - name: nex2426
        image: nex2426:latest
        ports:
        - containerPort: 8080
        env:
        - name: NEX2426_COST
          value: "3"
        - name: NEX2426_THREADS
          value: "4"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          exec:
            command:
            - nex2426
            - --version
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
            - nex2426
            - --version
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: config
          mountPath: /etc/nex2426
          readOnly: true
        - name: data
          mountPath: /var/lib/nex2426
      volumes:
      - name: config
        configMap:
          name: nex2426-config
      - name: data
        persistentVolumeClaim:
          claimName: nex2426-data
---
apiVersion: v1
kind: Service
metadata:
  name: nex2426-service
spec:
  selector:
    app: nex2426
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

---

## Monitoring and Maintenance

### Health Monitoring

```bash
# Create health check script
cat > /usr/local/bin/nex2426-health.sh << 'EOF'
#!/bin/bash

# Check if service is running
if ! systemctl is-active --quiet nex2426; then
    echo "ERROR: NEX2426 service is not running"
    exit 1
fi

# Check if binary is responsive
if ! /usr/local/bin/nex2426 --version >/dev/null 2>&1; then
    echo "ERROR: NEX2426 binary is not responding"
    exit 1
fi

# Check memory usage
MEMORY_USAGE=$(ps -o pid,vsz,rss,comm -p $(pgrep nex2426) | tail -1 | awk '{print $3}')
if [ "$MEMORY_USAGE" -gt 1048576 ]; then  # 1GB
    echo "WARNING: High memory usage: ${MEMORY_USAGE}KB"
fi

# Check disk space
DISK_USAGE=$(df /var/lib/nex2426 | tail -1 | awk '{print $5}' | sed 's/%//')
if [ "$DISK_USAGE" -gt 90 ]; then
    echo "WARNING: High disk usage: ${DISK_USAGE}%"
fi

echo "OK: NEX2426 is healthy"
exit 0
EOF

chmod +x /usr/local/bin/nex2426-health.sh
```

### Log Monitoring

```bash
# Configure log rotation
sudo tee /etc/logrotate.d/nex2426 << 'EOF'
/var/log/nex2426/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 nex2426 nex2426
    postrotate
        systemctl reload nex2426
    endscript
}
EOF

# Monitor logs in real-time
tail -f /var/log/nex2426/nex2426.log
```

### Performance Monitoring

```bash
# Create performance monitoring script
cat > /usr/local/bin/nex2426-monitor.sh << 'EOF'
#!/bin/bash

# Run benchmark and log results
BENCHMARK_RESULT=$(/usr/local/bin/nex2426 --bench 3 | grep "Hashes/Second" | awk '{print $3}')
TIMESTAMP=$(date -Iseconds)

echo "${TIMESTAMP},${BENCHMARK_RESULT}" >> /var/log/nex2426/performance.log

# Alert if performance is degraded
if [ "$BENCHMARK_RESULT" -lt 1000 ]; then
    echo "ALERT: Low performance detected: ${BENCHMARK_RESULT} hashes/sec"
fi
EOF

chmod +x /usr/local/bin/nex2426-monitor.sh
```

### Automated Maintenance

```bash
# Create cron job for daily maintenance
sudo tee /etc/cron.d/nex2426-maintenance << 'EOF'
# NEX2426 maintenance tasks
0 2 * * * nex2426 /usr/local/bin/nex2426-monitor.sh
0 3 * * 0 root /usr/local/bin/nex2426-health.sh
0 4 * * * root logrotate -f /etc/logrotate.d/nex2426
EOF
```

---

## Security Considerations

### System Hardening

```bash
# Set secure file permissions
sudo chmod 755 /usr/local/bin/nex2426
sudo chmod 644 /etc/nex2426/config.toml
sudo chmod 600 /etc/nex2426/keys.toml

# Configure SELinux (if enabled)
sudo semanage fcontext -a -t bin_t "/usr/local/bin/nex2426"
sudo restorecon -v /usr/local/bin/nex2426

# Configure AppArmor (if enabled)
sudo aa-complain /usr/local/bin/nex2426
```

### Key Management

```bash
# Create key management directory
sudo mkdir -p /etc/nex2426/keys
sudo chmod 700 /etc/nex2426/keys
sudo chown nex2426:nex2426 /etc/nex2426/keys

# Generate master key (example)
sudo -u nex2426 openssl rand -hex 32 > /etc/nex2426/keys/master.key
sudo chmod 600 /etc/nex2426/keys/master.key
```

### Network Security

```bash
# Configure firewall rules
sudo ufw allow from 10.0.0.0/8 to any port 8080
sudo ufw deny 8080

# Configure fail2ban for NEX2426
sudo tee /etc/fail2ban/jail.d/nex2426.conf << 'EOF'
[nex2426]
enabled = true
port = 8080
filter = nex2426
logpath = /var/log/nex2426/nex2426.log
maxretry = 5
bantime = 3600
EOF
```

---

## Troubleshooting

### Common Issues

#### 1. Build Failures

```bash
# Check Rust version
rustc --version

# Update Rust
rustup update

# Clean build
cargo clean
cargo build --release
```

#### 2. Performance Issues

```bash
# Check system resources
htop
free -h
iostat

# Check AVX2 support
grep avx2 /proc/cpuinfo

# Optimize configuration
sudo nano /etc/nex2426/config.toml
```

#### 3. Service Failures

```bash
# Check service status
sudo systemctl status nex2426

# Check logs
sudo journalctl -u nex2426 -f

# Restart service
sudo systemctl restart nex2426
```

#### 4. Permission Issues

```bash
# Check file permissions
ls -la /usr/local/bin/nex2426
ls -la /etc/nex2426/

# Fix permissions
sudo chown -R nex2426:nex2426 /var/lib/nex2426
sudo chmod +x /usr/local/bin/nex2426
```

### Debug Mode

```bash
# Enable debug logging
export NEX2426_LOG_LEVEL=debug

# Run with verbose output
nex2426 --verbose "test" "key" 1

# Check configuration
nex2426 --config-check
```

### Performance Profiling

```bash
# Profile with perf
sudo perf record -g /usr/local/bin/nex2426 --bench 3
sudo perf report

# Profile with strace
strace -c /usr/local/bin/nex2426 "test" "key" 1

# Memory profiling
valgrind --tool=massif /usr/local/bin/nex2426 "test" "key" 1
```

---

## Backup and Recovery

### Configuration Backup

```bash
# Backup configuration
sudo tar -czf /backup/nex2426-config-$(date +%Y%m%d).tar.gz /etc/nex2426/

# Backup data
sudo tar -czf /backup/nex2426-data-$(date +%Y%m%d).tar.gz /var/lib/nex2426/

# Backup logs
sudo tar -czf /backup/nex2426-logs-$(date +%Y%m%d).tar.gz /var/log/nex2426/
```

### Disaster Recovery

```bash
# Restore configuration
sudo tar -xzf /backup/nex2426-config-20240128.tar.gz -C /

# Restore data
sudo tar -xzf /backup/nex2426-data-20240128.tar.gz -C /

# Restore permissions
sudo chown -R nex2426:nex2426 /var/lib/nex2426
sudo systemctl restart nex2426
```

---

## Upgrade Procedures

### Minor Version Upgrade

```bash
# Backup current installation
sudo cp /usr/local/bin/nex2426 /usr/local/bin/nex2426.backup

# Download and install new version
wget https://github.com/nex2426/nex2426/releases/latest/download/nex2426-linux-amd64
sudo mv nex2426-linux-amd64 /usr/local/bin/nex2426
sudo chmod +x /usr/local/bin/nex2426

# Test new version
nex2426 --version

# Restart service
sudo systemctl restart nex2426
```

### Major Version Upgrade

```bash
# Full backup
sudo tar -czf /backup/nex2426-full-$(date +%Y%m%d).tar.gz \
    /usr/local/bin/nex2426 \
    /etc/nex2426 \
    /var/lib/nex2426 \
    /var/log/nex2426

# Stop service
sudo systemctl stop nex2426

# Run installation script
sudo ./install.sh

# Restore custom configuration
sudo cp /backup/nex2426-config-20240128.tar.gz /etc/nex2426/
sudo tar -xzf /backup/nex2426-config-20240128.tar.gz -C /

# Start service
sudo systemctl start nex2426
```

---

## Support and Resources

### Documentation

- **Main Documentation**: `/usr/local/share/doc/nex2426/README.md`
- **API Reference**: `/usr/local/share/doc/nex2426/API.md`
- **Security Guide**: `/usr/local/share/doc/nex2426/SECURITY.md`
- **Architecture**: `/usr/local/share/doc/nex2426/ARCHITECTURE.md`

### Community Support

- **GitHub Issues**: https://github.com/nex2426/nex2426/issues
- **Discussions**: https://github.com/nex2426/nex2426/discussions
- **Wiki**: https://github.com/nex2426/nex2426/wiki

### Professional Support

- **Email**: support@nex2426.io
- **Documentation**: https://docs.nex2426.io
- **Status Page**: https://status.nex2426.io

---

*This deployment guide covers the most common deployment scenarios. For specific requirements or custom deployments, consult the full documentation or contact the support team.*
