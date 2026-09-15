# coding-guidelines-v24 v6.37.0

## Quick Install (One-Liners)

### Windows (PowerShell 5.1+)
```powershell
irm https://github.com/alimtvnetwork/coding-guidelines-v24/releases/download/v6.37.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://github.com/alimtvnetwork/coding-guidelines-v24/releases/download/v6.37.0/install.sh | bash
```

## [v6.37.0] 2026-09-07 Modular BaseEnum Family, Coredata Combinators, and CI/CD Pipeline Expansion

### Install Coding Guidelines v6.37.0

Unix/Bash:
`curl -sL https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/v6.37.0/install.sh | bash -s -- ".lovable/prompts" "v6.37.0"`

PowerShell:
`Invoke-WebRequest -Uri https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/v6.37.0/install.ps1 -OutFile install.ps1; .\install.ps1 -TargetDir ".lovable/prompts" -Version "v6.37.0"`

### Added

- Modular BaseEnum family in `04-code/golang/pkg/baseenumer/`: `byte_enumer.go` (`ByteEnumer`, `ByteEnum`, `UTF8Enumer`, `UTF8Enum`), `utf16_enumer.go` (`UTF16Enumer`, `UTF16Enum`), `utf32_enumer.go` (`UTF32Enumer`, `UTF32Enum`, `RuneEnumer`, `RuneEnum`), `string_enumer.go` (`StringEnumer`, `StringEnum`), `number_enumer.go` (`NumberEnumer`, `NumberEnum`, `IntEnumer`, `IntEnum`).
- Coredata collection combinators on `ResultSlice[T]` (`Filter`, `ForEach`, `ForEachBreak`) in `04-code/golang/pkg/appfault/result_slice.go`.
- Coredata collection combinators on `ResultMap[K, V]` (`Keys`, `Values`, `Filter`, `ForEach`) in `04-code/golang/pkg/appfault/result_map.go`.
- Dynamic struct formatting and map inspection: `FormatStruct()` on `Result[T]`, `ResultSlice[T]`, `ResultMap[K, V]`, and `ToMap()` on `Result[T]`.
- Re-exported combinators in `04-code/golang/pkg/result/combinators.go`: `MapSlice`, `FlatMapSlice`, and `MapMapValues`.
- Unified Verifier Contracts: `SimpleVerifier` and `SimpleVerifyChecker` with `.AsSimpleVerifier()` and `.AsSimpleVerifyChecker()`.
- Expanded local CI/CD runner matrix (`03-ai-scripts/02-shared-engine.py`) to 31 concurrent quality gates.

---
