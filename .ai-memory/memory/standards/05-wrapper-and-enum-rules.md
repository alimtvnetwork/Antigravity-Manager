# Wrapper and Enum Standards

description: Enforces query wrapper logic, zero-tolerance for string unions, strict Enum naming, and positive boolean checks for failure cases.

## 1. Query Wrappers & Error Management

- **Automatic Failure Logging**: All database queries (across PHP, Python, and TypeScript) MUST be wrapped in a query execution utility that automatically catches and logs failures via the central logger before bubbling the error up.
- **No Manual `$isFailed = !$exists` Assignments**: Avoid unnecessary variable assignments for simple query checks. Direct wrapper logic should handle execution state tracking cleaner (e.g., `$wrapper->exists()` or `$query->fetchOrFail()`).

## 2. Zero-Tolerance for String Unions

- TypeScript string unions (e.g., `"pass" | "fail" | "fallback"`) are strictly forbidden.
- Any domain value that implies a status, category, or fixed set of identifiers MUST be extracted into a strictly typed Enum.

## 3. Strict Enum Naming

- Every Enum identifier MUST end with the suffix `Type` (e.g., `StatusType`, `LogLevelType`).
- Names like `Status7` or raw `Status` are forbidden for Enum definitions.

## 4. Positive Failure Checks

- Inverting a success boolean (`!response.isSuccess`) is considered bad code quality.
- Use explicit failure properties instead. Reverse all `!response.isSuccess` to `response.isFail` or `response.isFailed`.

## 5. No Magic Strings or Numbers

- Magic strings and numbers are strictly forbidden in business logic, conditional checks, and parameters, UNLESS they are explicitly within a logger method call.
