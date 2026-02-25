# tmui

[English](README.md)

tmux 세션을 관리하는 빠르고 vim 키 기반의 TUI. Rust, [ratatui](https://ratatui.rs), [nucleo](https://github.com/helix-editor/nucleo) 퍼지 매칭으로 구현되었습니다.

## 기능

- **세션 목록** - 패인 내용 실시간 미리보기 (ANSI 컬러 지원)
- **Vim 스타일 탐색** (`j`/`k`, `G`/`gg`)
- **퍼지 검색** (`/`) - nucleo-matcher 기반, 매칭 하이라이트
- **세션 태깅** 및 태그 기반 필터링
- **윈도우 확장** (Tab) - 세션 내 윈도우 확인
- **도움말 오버레이** (`?`) - 키바인딩 치트시트
- **CJK/유니코드 지원** - 세션 이름 및 미리보기

## 설치

```bash
git clone https://github.com/sm-kim1/tmui.git
cd tmui
./install.sh
```

`install.sh`는 다음을 수행합니다:

1. `cargo build --release`로 바이너리 빌드
2. `~/.cargo/bin/`에 바이너리 설치
3. `.tmux.conf`를 `~/`에 복사 (기존 파일은 `.tmux.conf.bak`으로 백업)
4. tmux 실행 중이면 설정 자동 리로드

### 요구사항

- Rust 1.70+
- tmux (최신 버전 권장)

## tmux 연동

설치 후 tmux에서 `prefix + s`를 누르면 기본 세션 목록(`choose-tree`) 대신 tmui가 팝업으로 실행됩니다.

```tmux
# .tmux.conf
bind s display-popup -E -w 80% -h 80% tmui
```

> 기본 prefix는 `Ctrl + a`입니다.

## 사용법

```bash
# tmux 안팎에서 실행 가능
tmui
```

### 키바인딩

| 키      | 동작                     |
|---------|--------------------------|
| `j`/`k` | 아래/위 이동             |
| `G`     | 마지막으로 이동           |
| `gg`    | 처음으로 이동             |
| `Enter` | 세션 연결/전환            |
| `n`     | 새 세션 생성              |
| `r`     | 세션 이름 변경            |
| `dd`    | 세션 종료 (확인)          |
| `D`     | 클라이언트 분리           |
| `/`     | 퍼지 검색                |
| `t`     | 세션에 태그 추가          |
| `T`     | 태그로 필터 / 해제        |
| `Tab`   | 윈도우 펼치기/접기        |
| `?`     | 도움말 토글              |
| `q`     | 종료                     |

### tmux 안에서 vs 밖에서

- **tmux 안**: `switch-client`로 세션 전환
- **tmux 밖**: `attach-session`으로 프로세스를 대체하여 연결

## 설정

설정 파일은 `~/.config/tmui/config.toml` (XDG)에 저장됩니다. 태그와 그룹은 자동으로 유지됩니다.

```toml
[tags]
work = ["important", "dev"]
personal = ["home"]

[groups]
```

## 개발

```bash
cargo test          # 73개 이상의 테스트 실행
cargo clippy        # 린트 (경고 0개)
cargo fmt --check   # 포맷 검사
```

## 라이선스

MIT
