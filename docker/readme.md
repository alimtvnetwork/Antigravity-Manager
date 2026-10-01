# 🐋 Antigravity Manager Native Docker Deployment Guide

This directory contains the native Headless Docker deployment solution for Antigravity Manager. It provides a complete Web management UI, high-performance API proxy gateway, and full data persistence without requiring complex VNC or desktop GUI environments.

## 🆕 Deployment Strategy (Local Frontend Build Reuse)
Ideal for scenarios where the backend changes frequently while the frontend remains stable. By generating `dist/` locally first, Docker only compiles the Rust backend and copies `dist/`, dramatically reducing build times and eliminating frontend build risks.

**Steps**
1. Generate frontend static assets locally:
```bash
npm ci --legacy-peer-deps
npm run build
```
2. Build and launch using local dist reuse (backend-only compilation):
```bash
docker compose -f docker/docker-compose.yml -f docker/docker-compose.localdist.yml build
docker compose -f docker/docker-compose.yml -f docker/docker-compose.localdist.yml up -d
```
Or combine into a single command:
```bash
docker compose -f docker/docker-compose.yml -f docker/docker-compose.localdist.yml up -d --build
```

View dynamic logs after launch:
```bash
docker compose -f docker/docker-compose.yml -f docker/docker-compose.localdist.yml logs -f --tail=200
```

**Updating**
- Backend changes: Re-run `build` + `up -d` above.
- Frontend changes: Run `npm run build` locally first, then re-run `build` + `up -d`.

**Git Deployment Note**
- If deploying to a remote server that does not build the frontend, ensure `dist/` is committed to the repository (removed from `.gitignore` for this release).

## 🚀 Quick Start

### 1. Direct Image Pull (Recommended)
You can pull pre-built images directly from Docker Hub without cloning the source code:

> [!IMPORTANT]
> **Security Notice**: Starting from v4.0.3, Docker mode supports **separation of Admin Password and API Key**:
> * **API Key**: Set via `-e API_KEY=xxx`, used for AI protocol client requests.
> * **Web Password**: Set via `-e WEB_PASSWORD=xxx`, used exclusively for Web UI login.
> * **Default Behavior**: If `WEB_PASSWORD` is not set, the system falls back to `API_KEY` as the login password. If neither is set, a random key is generated.
> * **Inspection**: Run `docker logs antigravity-manager` to locate `Current API Key` or `Web UI Password`, or check `grep -E '"api_key"|"admin_password"' ~/.antigravity_tools/gui_config.json`.

```bash
# Start container (replace placeholder keys with strong secrets)
docker run -d \
  --name antigravity-manager \
  -p 8045:8045 \
  -e API_KEY=your-api-key \
  -e WEB_PASSWORD=your-login-password \
  -e ABV_MAX_BODY_SIZE=104857600 \
  -v ~/.antigravity_tools:/root/.antigravity_tools \
  alimtvnetwork/antigravity-manager:latest
```

> [!TIP]
> **🧪 Preview / Beta Releases**:
> To test the latest pre-release features, pull the corresponding Beta tag (pre-releases are built independently without overwriting the `latest` stable production tag):
> ```bash
> # Pull specific pre-release
> docker pull lbjlaq/antigravity-manager:v4.8.2-beta.0
> 
> # Run Beta container
> docker run -d --name antigravity-manager-beta \
>   -p 8045:8045 \
>   -e API_KEY=your-api-key \
>   -e WEB_PASSWORD=your-login-password \
>   -e ABV_MAX_BODY_SIZE=104857600 \
>   -v ~/.antigravity_tools:/root/.antigravity_tools \
>   lbjlaq/antigravity-manager:v4.8.2-beta.0
> ```
> For complete tag listings, visit [Docker Hub Tags](https://hub.docker.com/r/lbjlaq/antigravity-manager/tags). To build directly from the latest unreleased `beta` branch, run `docker build -t lbjlaq/antigravity-manager:beta -f docker/Dockerfile .`.

#### 🔐 Authentication Logic (Security Scenarios)
* **Scenario A: Only `API_KEY` is set**
  - **Web Login**: Access the Web UI using `API_KEY`.
  - **API Calls**: Authenticate AI protocol calls using `API_KEY`.
* **Scenario B: Both `API_KEY` and `WEB_PASSWORD` are set (Recommended)**
  - **Web Login**: **Must** use `WEB_PASSWORD`. Entering the API Key will be rejected, ensuring admin privileges are isolated from API consumers.
  - **API Calls**: Continue using `API_KEY`. You can safely distribute API keys to team members while keeping the admin password private.

#### 🆙 Upgrading from Older Versions
If upgrading from an older version without `WEB_PASSWORD` configured:
1. **Web UI (Recommended)**: Log in with the existing `API_KEY`, and set a new admin password in the **API Proxy** settings.
2. **Environment Variable**: Stop the old container, and add `-e WEB_PASSWORD=your-new-password` when starting the new container.

> [!TIP]
> **Priority Hierarchy**:
> - **Environment Variables** (`ABV_WEB_PASSWORD` / `WEB_PASSWORD`) take highest precedence. If set, the application will always use them, overriding configuration files.
> - **Configuration File** (`gui_config.json`) is used for persistence. When modified via the Web UI, the password is saved here under `admin_password`.
> - **Fallback**: If neither is set, fallback to `API_KEY`; if even `API_KEY` is not set, a random key is generated.

### 2. Using Docker Compose
In the `docker` directory, run:
```bash
docker compose up -d
```

### 3. Manual Image Build (Developer / Custom Build)
If you need to modify code or customize the build, run from the project root:

**Windows PowerShell (Recommended)**
```powershell
# One-click build custom image (tags: antigravity-manager:local and version)
.\docker\build.ps1

# Regional network mirror acceleration
.\docker\build.ps1 -UseMirror

# Build and push to your container registry
.\docker\build.ps1 -UseMirror -Push -Registry "yourname/antigravity-manager"
```

**Manual docker build**
```bash
# Default build local tag
docker build -t antigravity-manager:local -f docker/Dockerfile .

# Launch with custom compose (uses bridge port mapping for Windows compatibility)
docker compose -f docker/docker-compose.yml -f docker/docker-compose.fork.yml up -d --build
```

#### 💡 Build Arguments
This image supports automatic mirror switching to speed up builds:
* `USE_MIRROR`:
  - `auto` (Default): Automatically checks network connectivity; switches to mirror if Google is unreachable.
  - `true`: Force mirror sources.
  - `false`: Force official upstream sources.

Example:
```bash
docker build --build-arg USE_MIRROR=true -t antigravity-manager:latest -f docker/Dockerfile .
```

## ⚙️ Environment Variables Configuration

| Variable | Default | Description |
| :--- | :--- | :--- |
| `PORT` | `8045` | Port the container listens on |
| `ABV_API_KEY` | - | **[Important]** Proxy API key required by AI clients (e.g., Claude Code) |
| `ABV_WEB_PASSWORD` | - | **[Security]** Web UI admin password. Falls back to API Key if not set |
| `ABV_MAX_BODY_SIZE` | `104857600` | **[Performance]** Maximum request body size in bytes (default 100MB, resolves 413 Payload Too Large on large images) |
| `LOG_LEVEL` | `info` | Log level (debug, info, warn, error) |
| `ABV_DIST_PATH` | `/app/dist` | Path to frontend static assets (pre-baked in Dockerfile) |
| `ABV_PUBLIC_URL` | - | Optional public URL for remote OAuth callback redirects |

## 📂 Data Persistence
Make sure to mount a host directory to `/root/.antigravity_tools` in the container; otherwise, accounts and configurations will be lost upon container restart.

## 🌐 Endpoints
* **Management UI**: [http://localhost:8045](http://localhost:8045)
* **API Base**: [http://localhost:8045/v1](http://localhost:8045/v1)

## 📦 Docker Hub Distribution (Recommended)
To push to your registry:
```bash
docker tag antigravity-manager:latest alimtvnetwork/antigravity-manager:latest
docker tag antigravity-manager:latest alimtvnetwork/antigravity-manager:4.83.0
docker push alimtvnetwork/antigravity-manager:latest
docker push alimtvnetwork/antigravity-manager:4.83.0
```
