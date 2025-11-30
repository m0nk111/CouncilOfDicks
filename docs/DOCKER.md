# Docker Deployment Guide

## Quick Start

### Option 1: Docker Compose (Recommended)

Includes both Council and Ollama servers:

```bash
# Start everything
docker-compose up -d

# View logs
docker-compose logs -f council

# Stop everything
docker-compose down
```

Access:
- **Web UI**: http://localhost:8080
- **API**: http://localhost:8080/api/*
- **WebSocket**: ws://localhost:8080/ws/chat
- **Ollama**: http://localhost:11434

### Option 2: Docker Only

If you already have Ollama running elsewhere:

```bash
# Build image
docker build -t council-of-dicks .

# Run container
docker run -d \
  --name council \
  -p 8080:8080 \
  -p 9001:9001 \
  -e OLLAMA_URL=http://192.168.1.5:11434 \
  -e OLLAMA_MODEL=qwen2.5-coder:7b \
  -v council-data:/app/data \
  council-of-dicks
```

### Option 3: Connect to Host Ollama

If Ollama is running on your host machine:

```bash
docker run -d \
  --name council \
  -p 8080:8080 \
  -e OLLAMA_URL=http://host.docker.internal:11434 \
  council-of-dicks
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `OLLAMA_URL` | `http://ollama:11434` | Ollama server URL |
| `OLLAMA_MODEL` | `qwen2.5-coder:7b` | Default AI model |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `DEBUG_ENABLED` | `false` | Enable debug mode |

### Ports

- **8080**: HTTP API + WebSocket
- **9001**: MCP server (optional)

### Volumes

- `/app/data`: Persistent storage (identity keys, database)

## Docker Compose Configuration

### With Ollama Included

```yaml
services:
  council:
    build: .
    ports:
      - "8080:8080"
    environment:
      - OLLAMA_URL=http://ollama:11434
    depends_on:
      - ollama
  
  ollama:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
```

### Connect to External Ollama

```yaml
services:
  council:
    build: .
    ports:
      - "8080:8080"
    environment:
      - OLLAMA_URL=http://192.168.1.5:11434
```

## Health Checks

Check if server is running:

```bash
# Direct
curl http://localhost:8080/health

# Docker
docker exec council curl -f http://localhost:8080/health
```

## Logs

```bash
# Follow logs
docker-compose logs -f council

# Last 100 lines
docker logs --tail 100 council

# With timestamps
docker logs -t council
```

## Troubleshooting

### Ollama Connection Issues

```bash
# Check Ollama is reachable
docker exec council curl -v http://ollama:11434/api/version

# Or from host
curl http://localhost:11434/api/version
```

### Container Won't Start

```bash
# Check logs
docker logs council

# Inspect container
docker inspect council

# Verify image built correctly
docker images | grep council
```

### Port Already in Use

```bash
# Change port mapping in docker-compose.yml
ports:
  - "3000:8080"  # Host:Container
```

Or with docker run:
```bash
docker run -p 3000:8080 council-of-dicks
```

## Production Deployment

### With Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name council.example.com;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### With SSL (Let's Encrypt)

```bash
# Install certbot
apt install certbot python3-certbot-nginx

# Get certificate
certbot --nginx -d council.example.com

# Auto-renewal
certbot renew --dry-run
```

### Resource Limits

```yaml
services:
  council:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

## Updating

```bash
# Pull latest changes
git pull

# Rebuild and restart
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

## Backup

```bash
# Backup data volume
docker run --rm \
  -v council-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/council-backup.tar.gz /data

# Restore
docker run --rm \
  -v council-data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/council-backup.tar.gz -C /
```

## Security Notes

1. **Change default ports** in production
2. **Use environment files** for secrets (don't commit .env)
3. **Enable authentication** when exposing publicly
4. **Use HTTPS** with proper certificates
5. **Firewall rules** - only expose necessary ports
6. **Regular updates** - keep Docker images up to date

## Support

Issues: https://github.com/m0nk111/CouncilOfDicks/issues
Docs: https://github.com/m0nk111/CouncilOfDicks/tree/main/docs
