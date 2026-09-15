# Rule: Error Management Architecture

1. **Go Error Type:** All Go packages returning structured error metadata MUST use `*appfault.AppError` from `04-code/golang/pkg/appfault`.
2. **Never Swallow Errors:** Every catch block or error branch must log the operation name, key inputs, and preserve the original error cause.
3. **Universal Envelope:** API endpoints and service interfaces return standard envelopes (`{ data, errors[], meta }`).
