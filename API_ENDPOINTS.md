# VK-Service API Endpoints

This document describes all available API endpoints for the VK-Service. Use this as a reference for the load balancer to route requests correctly.

## Base URL
```
https://your-service.run.app
```

## Authentication

### Protected Routes
Protected routes require the `X-KV-SECRET` header with the service secret key.

```http
X-KV-SECRET: your-secret-key
```

### Public Routes
Public routes do not require authentication headers.

---

## Endpoints

### 1. Health Check
**GET** `/api/v1/health`

**Description:** Check the health status of the service instance.

**Authentication:** Required (`X-KV-SECRET` header)

**Response:**
```json
{
  "status": "ok",
  "server_id": "uuid",
  "timestamp": "2025-12-15T16:00:00Z"
}
```

---

### 2. Get All Instances
**GET** `/api/v1/instances`

**Description:** Retrieve information about all service instances.

**Authentication:** Required (`X-KV-SECRET` header)

**Response:**
```json
[
  {
    "server_id": "uuid",
    "provider": "supabase",
    "created_at": "2025-12-15T16:00:00Z"
  }
]
```

---

### 3. Get Instance
**GET** `/api/v1/instances/{server_id}`

**Description:** Get detailed information about a specific service instance.

**Authentication:** Required (`X-KV-SECRET` header)

**Path Parameters:**
- `server_id` (string, UUID): The unique identifier of the server instance

**Response:**
```json
{
  "server_id": "uuid",
  "provider": "supabase",
  "created_at": "2025-12-15T16:00:00Z",
  "config": {
    "max_file_size": 524288000,
    "allowed_extensions": ["jpg", "png", "pdf"]
  }
}
```

---

### 4. Update Instance
**PATCH** `/api/v1/instances/{server_id}`

**Description:** Update configuration for a specific service instance.

**Authentication:** Required (`X-KV-SECRET` header)

**Path Parameters:**
- `server_id` (string, UUID): The unique identifier of the server instance

**Request Body:**
```json
{
  "provider": "supabase"
}
```

**Response:**
```json
{
  "server_id": "uuid",
  "provider": "supabase",
  "updated_at": "2025-12-15T16:00:00Z"
}
```

---

### 5. Create User
**POST** `/api/v1/users`

**Description:** Create a new user with storage quota.

**Authentication:** Not required

**Request Body:**
```json
{
  "uid": "user-uuid-optional"
}
```

**Response:**
```json
{
  "uid": "generated-or-provided-uuid",
  "file_count": 0,
  "total_space": 1073741824,
  "used_space": 0,
  "created_at": "2025-12-15T16:00:00Z"
}
```

**Notes:**
- If `uid` is not provided, a new UUID will be generated
- `total_space` is set to the default quota from global config (default: 1GB)

---

### 6. Get User
**GET** `/api/v1/users/{user_id}`

**Description:** Retrieve user information and storage usage.

**Authentication:** Not required

**Path Parameters:**
- `user_id` (string, UUID): The user's unique identifier

**Response:**
```json
{
  "uid": "user-uuid",
  "file_count": 5,
  "total_space": 1073741824,
  "used_space": 52428800,
  "created_at": "2025-12-15T16:00:00Z"
}
```

---

### 7. Update User
**PATCH** `/api/v1/users/{user_id}`

**Description:** Update user storage quota or usage information.

**Authentication:** Not required

**Path Parameters:**
- `user_id` (string, UUID): The user's unique identifier

**Request Body:**
```json
{
  "file_count": 10,
  "total_space": 2147483648,
  "used_space": 104857600
}
```

**Response:**
```json
{
  "uid": "user-uuid",
  "file_count": 10,
  "total_space": 2147483648,
  "used_space": 104857600,
  "updated_at": "2025-12-15T16:00:00Z"
}
```

**Notes:**
- All fields are optional
- Only provided fields will be updated

---

### 8. Delete User
**DELETE** `/api/v1/users/{user_id}`

**Description:** Delete a user and all associated files.

**Authentication:** Not required

**Path Parameters:**
- `user_id` (string, UUID): The user's unique identifier

**Response:**
```
204 No Content
```

**Notes:**
- This will delete all files associated with the user from storage
- This operation cannot be undone

---

### 9. Get User Files
**GET** `/api/v1/users/{user_id}/files`

**Description:** List all files belonging to a user.

**Authentication:** Not required

**Path Parameters:**
- `user_id` (string, UUID): The user's unique identifier

**Response:**
```json
[
  {
    "file_id": "1a2b3c4d5e6f7890",
    "filename": "document.pdf",
    "mime_type": "application/pdf",
    "size": 1048576,
    "provider": "supabase",
    "created_at": "2025-12-15T16:00:00Z"
  }
]
```

---

### 10. Generate Upload Token
**POST** `/api/v1/files/token`

**Description:** Generate a temporary token for file upload.

**Authentication:** Not required

**Request Body:**
```json
{
  "user_id": "user-uuid-optional"
}
```

**Response:**
```json
{
  "token": "jwt-token-string",
  "expires_at": "2025-12-15T17:00:00Z"
}
```

**Notes:**
- Token is valid for 1 hour
- If `user_id` is provided, the token will be associated with that user
- If `user_id` is not provided, the upload will be anonymous

---

### 11. Upload File
**POST** `/api/v1/files`

**Description:** Upload a file to storage.

**Authentication:** Not required (uses token from header)

**Headers:**
```http
Authorization: Bearer <upload-token>
Content-Type: multipart/form-data
```

**Request Body (multipart/form-data):**
- `file` (file): The file to upload

**Response:**
```json
{
  "file_id": "1a2b3c4d5e6f7890",
  "filename": "document.pdf",
  "mime_type": "application/pdf",
  "size": 1048576,
  "provider": "supabase",
  "user_id": "user-uuid-or-null",
  "created_at": "2025-12-15T16:00:00Z"
}
```

**Error Responses:**
- `400 Bad Request`: Missing or invalid file
- `401 Unauthorized`: Invalid or expired token
- `413 Payload Too Large`: File exceeds maximum size limit
- `507 Insufficient Storage`: User quota exceeded

---

### 12. Download File
**GET** `/api/v1/files/{file_id}/content`

**Description:** Download file content.

**Authentication:** Not required

**Path Parameters:**
- `file_id` (string): The unique file identifier

**Response:**
- Binary file content with appropriate `Content-Type` header

**Headers:**
```http
Content-Type: <file-mime-type>
Content-Disposition: attachment; filename="<original-filename>"
```

**Error Responses:**
- `404 Not Found`: File does not exist

---

### 13. Get File Metadata
**GET** `/api/v1/files/{file_id}`

**Description:** Retrieve file metadata without downloading content.

**Authentication:** Not required

**Path Parameters:**
- `file_id` (string): The unique file identifier

**Response:**
```json
{
  "file_id": "1a2b3c4d5e6f7890",
  "filename": "document.pdf",
  "mime_type": "application/pdf",
  "size": 1048576,
  "provider": "supabase",
  "user_id": "user-uuid-or-null",
  "created_at": "2025-12-15T16:00:00Z"
}
```

---

### 14. Update File Metadata
**PATCH** `/api/v1/files/{file_id}`

**Description:** Update file metadata (e.g., rename file).

**Authentication:** Not required

**Path Parameters:**
- `file_id` (string): The unique file identifier

**Request Body:**
```json
{
  "filename": "new-document-name.pdf"
}
```

**Response:**
```json
{
  "file_id": "1a2b3c4d5e6f7890",
  "filename": "new-document-name.pdf",
  "mime_type": "application/pdf",
  "size": 1048576,
  "provider": "supabase",
  "updated_at": "2025-12-15T16:00:00Z"
}
```

**Notes:**
- Only `filename` can be updated
- File content and `file_id` remain unchanged

---

### 15. Delete File
**DELETE** `/api/v1/files/{file_id}`

**Description:** Delete a file from storage.

**Authentication:** Not required

**Path Parameters:**
- `file_id` (string): The unique file identifier

**Response:**
```
204 No Content
```

**Notes:**
- File is permanently deleted from storage provider
- User's `file_count` and `used_space` are automatically decremented

---

### 16. Cleanup Expired Files
**DELETE** `/api/v1/files`

**Description:** Delete files with expired upload tokens (maintenance endpoint).

**Authentication:** Not required

**Response:**
```json
{
  "deleted_count": 5,
  "message": "Expired files cleaned up successfully"
}
```

**Notes:**
- This endpoint should be called periodically by a cron job
- Deletes files uploaded with anonymous tokens that have expired

---

## Storage Providers

The service supports multiple storage providers:

### Supabase Storage (S3-compatible)
- Uses AWS S3 SDK
- File IDs are hex-encoded timestamps without extensions
- Example file ID: `1a2b3c4d5e6f7890`

### Google Drive
- Uses Google Drive API
- File IDs are Google Drive native IDs
- Example file ID: `1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms`

---

## File ID Format

**Supabase Storage:**
- Format: Hexadecimal timestamp (no extension, no path separators)
- Example: `1a2b3c4d5e6f7890`
- Length: Variable (typically 16-20 characters)

**Google Drive:**
- Format: Google Drive document ID
- Example: `1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms`
- Length: Variable (typically 30-50 characters)

---

## Error Responses

All endpoints return standard HTTP status codes:

**Success:**
- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `204 No Content`: Request successful, no content to return

**Client Errors:**
- `400 Bad Request`: Invalid request body or parameters
- `401 Unauthorized`: Missing or invalid authentication
- `404 Not Found`: Resource not found
- `413 Payload Too Large`: Request body too large
- `507 Insufficient Storage`: Storage quota exceeded

**Server Errors:**
- `500 Internal Server Error`: Unexpected server error
- `503 Service Unavailable`: Service temporarily unavailable

**Error Response Format:**
```json
{
  "error": "Error message description",
  "code": "ERROR_CODE",
  "details": "Additional error details if available"
}
```

---

## Rate Limiting

Currently, no rate limiting is implemented. This should be handled at the load balancer level.

**Recommended limits:**
- File upload: 100 requests per hour per IP
- File download: 1000 requests per hour per IP
- API calls: 500 requests per hour per IP

---

## CORS

The service supports CORS with the following configuration:

- **Allowed Origins:** Configurable via `CORS_ALLOWED_ORIGINS` environment variable (comma-separated)
- **Allowed Methods:** All methods (GET, POST, PATCH, DELETE, OPTIONS)
- **Allowed Headers:** All headers

If `CORS_ALLOWED_ORIGINS` is not set, the service allows all origins (permissive mode - only for development).

---

## Environment Variables

The service requires the following environment variables:

- `SERVER_ID`: Unique identifier for this instance (UUID)
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `PORT`: Server port (default: 8080, auto-set by Cloud Run)
- `CORS_ALLOWED_ORIGINS`: Comma-separated list of allowed CORS origins (optional)

---

## Load Balancer Configuration

### Routing Strategy

**For file operations (upload/download):**
- Route based on storage provider preference
- Sticky sessions recommended for multi-part uploads
- Consider geographic proximity to storage provider

**For user operations:**
- Can route to any available instance
- No sticky sessions required

**For instance management:**
- Must route to specific instance based on `server_id`

### Health Check Configuration

```yaml
health_check:
  endpoint: /api/v1/health
  method: GET
  headers:
    X-KV-SECRET: your-secret-key
  interval: 30s
  timeout: 5s
  healthy_threshold: 2
  unhealthy_threshold: 3
```

### Example Load Balancer Rules

```yaml
# Route instance-specific requests
- path: /api/v1/instances/{server_id}
  target: instance-{server_id}

# Route file operations (round-robin or provider-based)
- path: /api/v1/files/*
  target: available-instances
  strategy: round-robin

# Route user operations (any instance)
- path: /api/v1/users/*
  target: available-instances
  strategy: least-connections
```

---

## Notes for Balancer Implementation

1. **Token Validation:** Upload tokens are stored in Redis and shared across all instances, so any instance can validate them.

2. **File Storage:** Files are stored in external storage (Supabase/GDrive), so any instance can retrieve them using the `file_id`.

3. **Database:** All instances share the same PostgreSQL database, ensuring data consistency.

4. **Session Affinity:** Not required for most operations, but recommended for large file uploads.

5. **Provider Selection:** Each instance can be configured to use a different storage provider. Route requests based on user preference or provider availability.
