# 기능

* 크로스 플랫폼 지원: Windows, Linux, macOS.
* 메모리 안전성, 빠른 속도, 독립 실행을 목표로 Rust로 개발.
* 멀티 스레드 성능으로 `.json` 또는 압축된 `.json.gz` 파일을 스캔.
* 포렌식 조사 및 사고 대응을 위한 분석하기 쉬운 단일 타임라인 생성.
* 읽기/생성/편집이 쉬운 YML 기반 [Sigma](https://github.com/SigmaHQ/sigma) 규칙으로 작성된 IoC 시그니처에 대한 뛰어난 네이티브 지원. ([expand](https://sigmahq.io/docs/basics/modifiers.html#expand)를 제외한 상관 규칙과 모든 필드 수정자가 지원됩니다.)
* 모든 API 사용 현황과 공격자에 대한 메트릭(소스 IP 주소, 지리적 위치, 사용된 리전, 사용자 에이전트 등)에 대한 요약을 생성하여 시그니처에 의존하지 않고 비정상적인 활동을 탐지.
* 결과를 CSV, JSON, JSONL로 저장.
