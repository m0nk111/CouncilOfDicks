#!/bin/bash
# Council Of Dicks - Installer Script
# Installs as systemd service with web UI on localhost:8080

set -e

INSTALL_DIR="/opt/council-of-dicks"
SERVICE_NAME="council-of-dicks"
USER="council"
CONFIG_DIR="/etc/council-of-dicks"
DATA_DIR="/var/lib/council-of-dicks"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "╔════════════════════════════════════════╗"
echo "║   Council Of Dicks - Installer v0.7.0  ║"
echo "╚════════════════════════════════════════╝"
echo -e "${NC}"

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}❌ Please run as root (sudo ./install.sh)${NC}"
    exit 1
fi

echo -e "${YELLOW}📦 Step 1/6: Checking dependencies...${NC}"

# Check for Ollama
if command -v ollama &> /dev/null; then
    echo -e "${GREEN}✅ Ollama is installed${NC}"
    OLLAMA_URL="http://localhost:11434"
else
    echo -e "${YELLOW}⚠️  Ollama not found. Installing...${NC}"
    curl -fsSL https://ollama.ai/install.sh | sh
    echo -e "${GREEN}✅ Ollama installed${NC}"
    OLLAMA_URL="http://localhost:11434"
    
    # Start Ollama service
    systemctl enable ollama
    systemctl start ollama
    
    # Pull default model
    echo -e "${YELLOW}📥 Downloading default AI model (qwen2.5-coder:7b)...${NC}"
    ollama pull qwen2.5-coder:7b
fi

echo -e "${YELLOW}👤 Step 2/6: Creating service user...${NC}"

# Create service user if not exists
if ! id -u $USER &> /dev/null; then
    useradd -r -s /bin/false -d $DATA_DIR $USER
    echo -e "${GREEN}✅ User '$USER' created${NC}"
else
    echo -e "${GREEN}✅ User '$USER' already exists${NC}"
fi

echo -e "${YELLOW}📁 Step 3/6: Creating directories...${NC}"

# Create directories
mkdir -p $INSTALL_DIR
mkdir -p $CONFIG_DIR
mkdir -p $DATA_DIR
mkdir -p $DATA_DIR/logs

# Copy binary
if [ -f "./src-tauri/target/release/app" ]; then
    cp ./src-tauri/target/release/app $INSTALL_DIR/council-server
    chmod +x $INSTALL_DIR/council-server
    echo -e "${GREEN}✅ Binary installed to $INSTALL_DIR${NC}"
else
    echo -e "${RED}❌ Binary not found. Run 'cargo build --release' first${NC}"
    exit 1
fi

# Copy web files
if [ -f "./test-web-mode.html" ]; then
    cp ./test-web-mode.html $INSTALL_DIR/
    cp ./test-websocket.html $INSTALL_DIR/
    echo -e "${GREEN}✅ Web UI files installed${NC}"
fi

echo -e "${YELLOW}⚙️  Step 4/6: Creating configuration...${NC}"

# Create config file
cat > $CONFIG_DIR/config.json <<EOF
{
  "ollama_url": "$OLLAMA_URL",
  "default_model": "qwen2.5-coder:7b",
  "debug_enabled": false,
  "http_server": {
    "host": "127.0.0.1",
    "port": 8080
  },
  "mcp_server": {
    "host": "127.0.0.1",
    "port": 9001
  }
}
EOF

echo -e "${GREEN}✅ Configuration created at $CONFIG_DIR/config.json${NC}"

echo -e "${YELLOW}🔧 Step 5/6: Creating systemd service...${NC}"

# Create systemd service file
cat > /etc/systemd/system/$SERVICE_NAME.service <<EOF
[Unit]
Description=Council Of Dicks - AI Consensus Network
Documentation=https://github.com/m0nk111/CouncilOfDicks
After=network-online.target ollama.service
Wants=network-online.target
Requires=ollama.service

[Service]
Type=simple
User=$USER
Group=$USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/council-server --server --host 127.0.0.1 --port 8080
Restart=on-failure
RestartSec=5s
StandardOutput=append:$DATA_DIR/logs/council.log
StandardError=append:$DATA_DIR/logs/council-error.log

# Environment
Environment="RUST_LOG=info"
Environment="COUNCIL_CONFIG=$CONFIG_DIR/config.json"
Environment="COUNCIL_DATA=$DATA_DIR"

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_DIR $CONFIG_DIR

[Install]
WantedBy=multi-user.target
EOF

echo -e "${GREEN}✅ Systemd service created${NC}"

# Set permissions
chown -R $USER:$USER $INSTALL_DIR
chown -R $USER:$USER $CONFIG_DIR
chown -R $USER:$USER $DATA_DIR

echo -e "${YELLOW}🚀 Step 6/6: Starting service...${NC}"

# Reload systemd
systemctl daemon-reload

# Enable and start service
systemctl enable $SERVICE_NAME
systemctl start $SERVICE_NAME

# Wait for service to start
sleep 2

# Check status
if systemctl is-active --quiet $SERVICE_NAME; then
    echo -e "${GREEN}"
    echo "╔════════════════════════════════════════╗"
    echo "║   ✅ Installation Complete!            ║"
    echo "╚════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
    echo -e "${GREEN}Service Status:${NC} $(systemctl is-active $SERVICE_NAME)"
    echo ""
    echo -e "${BLUE}📊 Web UI:${NC} http://localhost:8080"
    echo -e "${BLUE}🔧 Config:${NC} $CONFIG_DIR/config.json"
    echo -e "${BLUE}📁 Data:${NC} $DATA_DIR"
    echo -e "${BLUE}📜 Logs:${NC} $DATA_DIR/logs/"
    echo ""
    echo -e "${YELLOW}Useful commands:${NC}"
    echo "  sudo systemctl status $SERVICE_NAME   # Check status"
    echo "  sudo systemctl stop $SERVICE_NAME     # Stop service"
    echo "  sudo systemctl start $SERVICE_NAME    # Start service"
    echo "  sudo systemctl restart $SERVICE_NAME  # Restart service"
    echo "  sudo journalctl -u $SERVICE_NAME -f   # View logs"
    echo ""
    echo -e "${GREEN}🚀 Open http://localhost:8080 in your browser!${NC}"
else
    echo -e "${RED}❌ Service failed to start${NC}"
    echo "Check logs: sudo journalctl -u $SERVICE_NAME -n 50"
    exit 1
fi
