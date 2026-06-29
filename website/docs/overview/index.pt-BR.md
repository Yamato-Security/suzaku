# Sobre o Suzaku

Suzaku (朱雀) significa ["Pássaro Vermelhão"](https://en.wikipedia.org/wiki/Vermilion_Bird), que é um deus que voa acima das nuvens governando os céus do sul na [mitologia chinesa](https://en.wikipedia.org/wiki/Four_Holy_Beasts).

Suzaku é um gerador de linha do tempo para caça a ameaças e perícia forense rápida de logs de nuvem.
(Imagine o [Hayabusa](https://github.com/Yamato-Security/hayabusa), mas para logs de nuvem em vez de logs de eventos do Windows.)
Atualmente está em desenvolvimento ativo, com suporte nativo de detecção [Sigma](https://github.com/SigmaHQ/sigma) para logs do AWS CloudTrail.
Planejamos oferecer suporte também a logs do Azure e do GCP.

Com logs de nuvem, há milhares de chamadas de API diferentes e mais eventos do que qualquer pessoa conseguiria examinar manualmente.
O Suzaku foi projetado não apenas para encontrar os ataques em meio ao ruído, mas também para fornecer uma linha do tempo de DFIR que contém apenas os eventos e dados de que você precisa para realizar uma investigação de perícia forense rápida.
Você também pode criar resumos para descobrir rapidamente o que aconteceu em alto nível, identificar comportamentos anormais sem depender de assinaturas e encontrar facilmente palavras-chave como endereços IP, user agents, regiões, geolocalização etc., para pivotar e não perder nenhum evento que um atacante tenha realizado depois que você os descobrir.
