# agora-protocol

Wire DTOs, versioned by module (`v1`, `v2`, ...). Prices and quantities cross the boundary as
decimal strings and are converted to ticks/lots exactly once, in `convert`; the domain never
sees a decimal.

Rules:

- adding a field with a default is compatible; anything else is a new version;
- every DTO derives `utoipa::ToSchema` so the OpenAPI document is generated, not hand-written;
- snapshot tests (`insta`) pin the JSON shape of every message.
