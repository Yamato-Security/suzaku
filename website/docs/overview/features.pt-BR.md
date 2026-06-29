# Funcionalidades

* Suporte multiplataforma: Windows, Linux, macOS.
* Desenvolvido em Rust para ser seguro em memória, rápido e independente.
* Analise arquivos `.json` ou arquivos compactados `.json.gz` com desempenho multithread.
* Crie linhas do tempo únicas e fáceis de analisar para investigações forenses e resposta a incidentes.
* Excelente suporte nativo para assinaturas de IoC escritas em regras [Sigma](https://github.com/SigmaHQ/sigma) baseadas em YML, fáceis de ler/criar/editar. (Regras de correlação e todos os modificadores de campo, exceto [expand](https://sigmahq.io/docs/basics/modifiers.html#expand), são suportados.)
* Crie um resumo de todo o uso da API, métricas sobre o atacante (endereços IP de origem, geolocalização, regiões utilizadas, user agents, etc...) para descobrir atividades anormais sem depender de assinaturas.
* Salve os resultados em CSV, JSON e JSONL.
