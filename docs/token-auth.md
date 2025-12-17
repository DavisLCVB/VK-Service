# Token de subida de archivos

## Endpoints
- POST /api/v1/files/token — genera token de un solo uso (TTL 5 min); body opcional `{ "userId": "uuid" }`.
- POST /api/v1/files — sube el archivo multipart usando el token (un solo uso).

## Cabeceras admitidas
- `Authorization: Bearer <token>` (nuevo, preferido).
- `X-Upload-Token: <token>` (compatibilidad).

## Cuerpo de subida (multipart/form-data)
- `file` (binario) — requerido.
- `filename` — requerido.
- `mime_type` — requerido y validado contra la lista permitida.
- `type` — requerido, `temporal` o `permanent`.
- `user_id` — requerido si `type = permanent`.
- `description` — opcional.

## Notas de balanceador
- Reenviar sin modificar la cabecera `Authorization` hacia el backend; si no es posible, mapearla a `X-Upload-Token` para compatibilidad.
- No cachear respuestas de `/api/v1/files/token` ni `/api/v1/files`.
- Asegurar que el tráfico sea HTTPS del cliente al balancer; si se termina TLS en el balancer, mantener el canal seguro hasta el backend o en una red interna confiable.

## Ejemplos
Generar token:
```bash
curl -X POST https://<host>/api/v1/files/token \
  -H "Content-Type: application/json" \
  -d '{"userId":"<uuid-opcional>"}'
```

Subir usando Authorization:
```bash
curl -X POST https://<host>/api/v1/files \
  -H "Authorization: Bearer <token>" \
  -F "file=@/path/al/archivo" \
  -F "filename=archivo.pdf" \
  -F "mime_type=application/pdf" \
  -F "type=permanent" \
  -F "user_id=<uuid>" \
  -F "description=opcional"
```

Subir usando cabecera legada:
```bash
curl -X POST https://<host>/api/v1/files \
  -H "X-Upload-Token: <token>" \
  -F "file=@/path/al/archivo" \
  -F "filename=archivo.pdf" \
  -F "mime_type=application/pdf" \
  -F "type=permanent" \
  -F "user_id=<uuid>"
```
