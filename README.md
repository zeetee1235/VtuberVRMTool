
- `VRM/VRMClothingMergerTool.cs`
  - 아바타 루트와 의상 루트를 입력받아, 의상 본/SkinnedMeshRenderer를 아바타로 병합하는 도구
- `VRM/MissingScriptCleaner.cs`
  - 선택한 오브젝트/프리팹에서 Missing MonoBehaviour를 제거하는 도구



## Nix

### 1) Flakes 활성화 (최초 1회)

`~/.config/nix/nix.conf`에 아래 설정이 있어야 합니다.

```conf
experimental-features = nix-command flakes
```

### 2) 개발 쉘 진입

```bash
nix develop
```

또는(비 flake 방식):

```bash
nix-shell
```

개발 쉘에는 아래 도구가 포함됩니다.

- `dotnet-sdk_8`
- `mono`
- `msbuild`
- `git`
- `ripgrep`
- `tree`

## use in Unity

1. 이 저장소의 스크립트를 Unity 프로젝트의 `Assets/Editor/` 아래로 복사.
2. Unity 상단 메뉴에서 아래 항목을 실행.
   - `Tools/VRM/옷 입히는 툴`
   - `Tools/VRM/Missing 스크립트 삭제`


### VRMClothingMergerTool

- 이름 기준으로 아바타 본을 찾아 의상 본을 재부모화
- 의상 본/SkinnedMeshRenderer 이름에 접미사 일괄 추가
- 의상 SkinnedMeshRenderer를 아바타 루트 하위로 이동
- 조건 충족 시 의상 루트 및 빈 부모 정리

### MissingScriptCleaner

- 선택된 씬 오브젝트/프리팹의 Missing 스크립트 탐지 및 제거
- 기본은 `GameObjectUtility.RemoveMonoBehavioursWithMissingScript` 사용
- 실패 시 `SerializedObject` 경로로 폴백 처리

## 디렉터리 구조

```text
.
├── VRM
│   ├── MissingScriptCleaner
│   └── VRMClothingMergerTool.cs
├── flake.nix
└── shell.nix
```
