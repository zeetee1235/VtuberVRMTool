# VtuberVRMTool

Unity Editor에서 VRM 아바타 작업을 보조하는 스크립트와 Rust TUI 도구 모음입니다.

## 구성 요소

- `VRM/VRMClothingMergerTool.cs`
  - 의상 본/SkinnedMeshRenderer를 아바타로 병합하는 Unity Editor 툴
- `VRM/MissingScriptCleaner.cs`
  - 선택한 오브젝트/프리팹의 Missing 스크립트를 정리하는 Unity Editor 툴
- `VRM/VRMRustTuiBridge.cs`
  - Unity 데이터를 JSON으로 내보내고 Rust TUI 실행/결과 로드를 돕는 브리지 창
- `tools/vrm-tui`
  - 터미널에서 실행하는 Rust TUI 도구(검사/드라이런 계획)

## Nix 개발 환경

### 1) Flakes 활성화

`~/.config/nix/nix.conf`

```conf
experimental-features = nix-command flakes
```

### 2) 개발 쉘 진입

```bash
nix develop
```

또는

```bash
nix-shell
```

### 3) 포함된 도구

- `dotnet-sdk_8`
- `mono`
- `rustc`
- `cargo`
- `clippy`
- `rustfmt`
- `git`
- `ripgrep`
- `tree`

## Rust TUI 실행

```bash
cd tools/vrm-tui
cargo run -- --input ../../Temp/vrm_tui_input.json --output ../../Temp/vrm_tui_report.json
```

키 조작:
- `q`: 종료
- `↑/↓`: 작업 선택
- `Enter`: 선택 작업 실행
- `s`: 결과 JSON 저장

## Unity에서 사용

1. Unity 프로젝트 `Assets/Editor/` 아래에 `VRM` 스크립트를 배치
2. Unity 메뉴 실행
   - `Tools/VRM/옷 입히는 툴`
   - `Tools/VRM/Missing 스크립트 삭제`
   - `Tools/VRM/Rust TUI 브리지`

## Rust TUI 브리지 사용 순서

1. Unity에서 `Tools/VRM/Rust TUI 브리지` 창 열기
2. 아바타 루트/의상 루트 지정 후 `1) 입력 JSON 내보내기`
3. `2) TUI 실행 명령 복사`로 터미널 실행 명령 복사
4. 터미널에서 명령 실행 후 TUI에서 분석(Enter) 및 저장(s)
5. Unity로 돌아와 `3) 결과 JSON 불러오기`

## 디렉터리

```text
.
├── VRM
│   ├── MissingScriptCleaner.cs
│   ├── VRMRustTuiBridge.cs
│   └── VRMClothingMergerTool.cs
├── tools
│   └── vrm-tui
│       ├── Cargo.toml
│       └── src/main.rs
├── flake.nix
├── flake.lock
└── shell.nix
```
