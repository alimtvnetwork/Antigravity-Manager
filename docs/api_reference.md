# API Reference (v4.3.0)

This document details the HTTP API interface exposed by **Antigravity Tools**.

> **Note**: In the v4.0.1 version, all services (including AI reverse generation and system management) have been integrated into the unified port **8045**. The original port 19527 has been deprecated.

## 1. Overview

Antigravity Gateway is a dual-role server:
1. **AI Proxy Interface**: Standard interface compatible with OpenAI/Anthropic/Google official SDK.
2. **Management Admin API**: RESTful interface used to manage accounts, configure systems, and monitor traffic.

### Authentication system

| Interface type | path prefix | Authentication method | Header example | illustrate |
| :--- | :--- | :--- | :--- | :--- |
| **AI Protocol** | `/v1/*`, `/v1beta/*` | API Key | `Authorization: Bearer <API_KEY>` | For AI client calls |
| **Admin API** | `/api/*` | Admin Token | `x-admin-token: <TOKEN>` | Used to manage background or script control |

> **Tip**: By default, the `Admin Token` and the `API Key` are the same value (i.e. the `API_KEY` you set in the `.env` or Docker environment variable).

---

## 2. Management API

**Base URL**: `http://<host>:8045/api`

### 2.1 Account Management

| method | path | illustrate | Parameter example |
| :--- | :--- | :--- | :--- |
| **GET** | `/accounts` | Get account list | - |
| **GET** | `/accounts/current` | Get the current active account | - |
| **POST** | `/accounts` | Add account (OAuth Refresh Token) | `{"refreshToken": "..."}` |
| **DELETE**| `/accounts/:id` | Delete account | - |
| **POST** | `/accounts/switch` | Switch active account | `{"accountId": "acc_123", "targetIde": "agy"}` (targetIde is optional, if you pass `"agy"`, only the credentials will be written without restarting the IDE) |
| **POST** | `/accounts/refresh` | **Refresh all account quotas** | - |
| **GET** | `/accounts/:id/quota` | **Query specific account quota** | - |
| **POST** | `/accounts/:id/toggle-proxy` | Disable/enable account proxy | - |
| **POST** | `/accounts/:id/bind-device` | Bind device fingerprint | `{"mode": "generate"}` |
| **POST** | `/accounts/bulk-delete` | Delete accounts in batches | `{"accountIds": ["id1", "id2"]}` |
| **POST** | `/accounts/reorder` | Account sorting | `{"accountIds": [...]}` |

### 2.2 System Config
| method | path | illustrate |
| :--- | :--- | :--- |
| **GET** | `/config` | Get full configuration |
| **POST** | `/config` | Save full configuration |
| **GET** | `/proxy/status` | Get the running status of the anti-generation service |
| **POST** | `/proxy/start` | Start anti-generation service |
| **POST** | `/proxy/stop` | Stop anti-generation service |
| **POST** | `/proxy/mapping` | Update model mapping rules |
| **GET** | `/health` | System health check |

### 2.3 Monitoring & Stats
#### Traffic log
* **GET** `/logs`: Get the log list (supports `limit`, `offset`, `filter`, `errorsOnly` parameters)
* **GET** `/logs/count`: Get the total number of logs
* **GET** `/logs/:id`: Get log details
* **POST** `/logs/clear`: Clear logs

#### Token statistics (v4.0.1 New)
* **GET** `/stats/token/summary`: Get the Token consumption summary (today/this week/total amount)
* **GET** `/stats/token/hourly`: Get hourly statistics
* **GET** `/stats/token/daily`: Get daily statistical data
* **GET** `/stats/token/by-account`: Statistics consumption ratio by account
* **GET** `/stats/token/by-model`: Statistical consumption ratio by model
* **POST** `/stats/token/clear`: Reset statistics

### 2.4 Advanced functions (Advanced)
* **POST** `/proxy/cli/sync`: Execute CLI (Claude/Codex) configuration file synchronization
* **POST** `/accounts/import/db`: Import accounts from v1 old database
* **POST** `/accounts/oauth/start`: Initiate the OAuth authorization process (Headless)
* **POST** `/proxy/cloudflared/start`: Start Cloudflare Tunnel

---

## 3. AI Protocol Interface

**Base URL**: `http://<host>:8045`

This service is fully compatible with the official protocol specifications of mainstream AI manufacturers. You can directly fill in the address of this service into the client that supports OpenAI / Claude.

### OpenAI Compatible
* **Chat Completions**
    *   **POST** `/v1/chat/completions`
    * **Supported models**: any mapped model ID (e.g. `gpt-4o`, `gemini-1.5-pro`)
    * **Compatibility**: Fully compatible with OpenAI official Response format (including streaming SSE).

* **Image Generation**
    *   **POST** `/v1/images/generations`
    * **Supported models**: `gemini-3-pro-image` (automatically mapped to Imagen 3)
    * **Parameter expansion**: Supports advanced parameters such as `size: "1920x1080"`, `quality: "hd"`.

### Anthropic Compatible
*   **Claude Messages**
    *   **POST** `/v1/messages`
    * **Use**: Support Claude CLI (`claude`), Cursor, Cherry Studio and other clients.
    * **Features**: Full support for Tool Use (tool calling) and Thinking (thinking chain) modes.

### Gemini Native
*   **Google AI Studio**
    *   **GET/POST** `/v1beta/models/*`
    * **Purpose**: Called by applications using Google’s official SDK (Python/Node.js).
