# Suzaku 소개

Suzaku(朱雀)는 [중국 신화](https://en.wikipedia.org/wiki/Four_Holy_Beasts)에서 구름 위를 날며 남쪽 하늘을 다스리는 신인 ["주작"](https://en.wikipedia.org/wiki/Vermilion_Bird)을 의미합니다.

Suzaku는 클라우드 로그를 위한 위협 헌팅 및 빠른 포렌식 타임라인 생성기입니다.
(윈도우 이벤트 로그 대신 클라우드 로그를 다루는 [Hayabusa](https://github.com/Yamato-Security/hayabusa)를 상상해 보세요.)
현재 AWS CloudTrail 로그에 대한 네이티브 [Sigma](https://github.com/SigmaHQ/sigma) 탐지 지원과 함께 활발히 개발 중입니다.
Azure 및 GCP 로그도 지원할 계획입니다.

클라우드 로그에는 수천 가지의 서로 다른 API 호출이 있으며, 누구도 수작업으로 일일이 살펴볼 수 없을 만큼 많은 이벤트가 존재합니다.
Suzaku는 잡음 속에서 공격을 찾아낼 뿐만 아니라, 빠른 포렌식 조사를 수행하는 데 필요한 이벤트와 데이터만을 담은 DFIR 타임라인을 제공하도록 설계되었습니다.
또한 무슨 일이 있었는지 상위 수준에서 빠르게 파악할 수 있도록 요약을 생성하고, 시그니처에 의존하지 않고 비정상적인 행위를 발견하며, IP 주소, 사용자 에이전트, 리전, 지리적 위치 등과 같은 키워드를 쉽게 찾아내어 이를 기준으로 피벗하고, 공격자를 발견한 후에 그가 수행한 어떤 이벤트도 놓치지 않을 수 있습니다.
